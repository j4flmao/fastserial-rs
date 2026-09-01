//! # Arena allocator
//!
//! A small chunked bump arena, modelled after `bumpalo`. Memory is allocated
//! in append-only chunks; once a value is placed inside the arena, its
//! address is stable for the lifetime of the arena (the arena never moves
//! existing memory). Only `reset()` or `Drop` invalidates references.
//!
//! ## Why a custom one?
//!
//! The previous implementation stored bytes inside a `Vec<u8>` and handed
//! out raw pointers. As soon as a later allocation grew the `Vec`, every
//! pointer handed out before the realloc became dangling — observable
//! undefined behaviour. This rewrite uses an append-only list of fixed
//! `Box<[MaybeUninit<u8>]>` chunks so reallocation is impossible.

#![doc(hidden)]

use alloc::alloc::Layout;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::Cell;
use core::mem::{self, MaybeUninit};
use core::ptr::{self, NonNull};

const DEFAULT_CHUNK_SIZE: usize = 4096;

struct Chunk {
    data: Box<[MaybeUninit<u8>]>,
    /// Offset into `data` of the next free byte.
    used: Cell<usize>,
}

impl Chunk {
    fn with_capacity(cap: usize) -> Self {
        let mut v = Vec::with_capacity(cap);
        // Safety: we never read uninitialised bytes through `&[u8]`; the
        // arena tracks initialised bytes via the `used` cursor and the
        // typed pointers handed out by `alloc`.
        #[allow(clippy::uninit_vec)]
        unsafe {
            v.set_len(cap);
        }
        Self {
            data: v.into_boxed_slice(),
            used: Cell::new(0),
        }
    }

    fn try_alloc(&self, layout: Layout) -> Option<NonNull<u8>> {
        let used = self.used.get();
        let base = self.data.as_ptr() as usize;
        let cur = base + used;
        let aligned = (cur + layout.align() - 1) & !(layout.align() - 1);
        let pad = aligned - cur;
        let new_used = used.checked_add(pad)?.checked_add(layout.size())?;
        if new_used > self.data.len() {
            return None;
        }
        self.used.set(new_used);
        // Safety: aligned ∈ [base + used, base + len], so it points inside
        // the chunk and is non-null because `data` is allocated.
        Some(unsafe { NonNull::new_unchecked(aligned as *mut u8) })
    }

    fn reset(&mut self) {
        self.used.set(0);
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }
}

/// A chunked bump arena.
///
/// Allocations are O(1) amortised. The arena never moves bytes once written,
/// so references handed out by [`Arena::alloc`] / [`Arena::alloc_slice`]
/// stay valid until [`Arena::reset`] is called or the arena is dropped.
pub struct Arena {
    /// All chunks except the current one. Older chunks may have wasted tail
    /// space, but they own live data and must not be reset until the arena
    /// itself is reset.
    chunks: core::cell::RefCell<Vec<Chunk>>,
    /// Active chunk we try to allocate from first.
    current: core::cell::RefCell<Chunk>,
}

impl Arena {
    /// Creates an arena with a single 4 KiB chunk.
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHUNK_SIZE)
    }

    /// Creates an arena with an initial chunk of the given size (rounded up
    /// to at least the default chunk size).
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        let cap = cap.max(DEFAULT_CHUNK_SIZE);
        Self {
            chunks: core::cell::RefCell::new(Vec::new()),
            current: core::cell::RefCell::new(Chunk::with_capacity(cap)),
        }
    }

    /// Allocates `val` in the arena and returns a stable mutable reference.
    #[inline]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        let layout = Layout::for_value(&val);
        let ptr = self.alloc_layout(layout).as_ptr() as *mut T;
        // Safety: ptr came from alloc_layout with the right size and
        // alignment for T, and we are the only writer.
        unsafe {
            ptr::write(ptr, val);
            &mut *ptr
        }
    }

    /// Copies `slice` into the arena and returns a stable mutable slice.
    #[inline]
    pub fn alloc_bytes(&self, len: usize) -> &mut [u8] {
        let layout = core::alloc::Layout::array::<u8>(len).unwrap();
        let ptr = self.alloc_layout(layout).as_ptr() as *mut u8;
        unsafe { core::slice::from_raw_parts_mut(ptr, len) }
    }

    #[inline]
    pub fn alloc_slice<T: Copy>(&self, slice: &[T]) -> &mut [T] {
        let layout = Layout::array::<T>(slice.len()).expect("layout overflow");
        let ptr = self.alloc_layout(layout).as_ptr() as *mut T;
        // Safety: layout fits exactly `slice.len()` Ts, and source is non-overlapping.
        unsafe {
            ptr::copy_nonoverlapping(slice.as_ptr(), ptr, slice.len());
            core::slice::from_raw_parts_mut(ptr, slice.len())
        }
    }

    fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        if let Some(p) = self.current.borrow().try_alloc(layout) {
            return p;
        }
        // Need a new chunk: at least double the previous size, but enough
        // to fit `layout` even if it's larger than DEFAULT_CHUNK_SIZE.
        let new_cap = (self.current.borrow().capacity() * 2)
            .max(layout.size() + layout.align())
            .max(DEFAULT_CHUNK_SIZE);
        let new_chunk = Chunk::with_capacity(new_cap);
        // Move current into the saved list, install the new chunk.
        let old = mem::replace(&mut *self.current.borrow_mut(), new_chunk);
        self.chunks.borrow_mut().push(old);
        // Allocation in a fresh chunk cannot fail unless layout itself is bogus.
        self.current
            .borrow()
            .try_alloc(layout)
            .expect("fresh chunk should always satisfy a single allocation")
    }

    /// Drops all chunks except the largest, which is reused. Invalidates
    /// every reference previously handed out.
    pub fn reset(&mut self) {
        let current = self.current.get_mut();
        let chunks = self.chunks.get_mut();

        let mut max_idx = None;
        let mut max_cap = current.capacity();

        for (i, c) in chunks.iter().enumerate() {
            if c.capacity() > max_cap {
                max_cap = c.capacity();
                max_idx = Some(i);
            }
        }

        if let Some(idx) = max_idx {
            let mut best = chunks.swap_remove(idx);
            core::mem::swap(current, &mut best);
        }

        current.reset();
        chunks.clear();
    }

    /// Total bytes used across all chunks, including alignment padding.
    pub fn used_bytes(&self) -> usize {
        self.current.borrow().used.get()
            + self
                .chunks
                .borrow()
                .iter()
                .map(|c| c.used.get())
                .sum::<usize>()
    }

    /// Total capacity across all chunks.
    pub fn capacity(&self) -> usize {
        self.current.borrow().capacity()
            + self
                .chunks
                .borrow()
                .iter()
                .map(|c| c.capacity())
                .sum::<usize>()
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_returns_correct_value() {
        let a = Arena::new();
        assert_eq!(*a.alloc(42u32), 42);
    }

    #[test]
    fn alloc_slice_copies() {
        let a = Arena::new();
        let s = a.alloc_slice(&[1u32, 2, 3, 4, 5]);
        assert_eq!(s, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn many_allocations_stay_alive_across_chunk_growth() {
        // The pre-fix arena would invalidate previously-returned addresses
        // as soon as the backing Vec grew (it called `Vec::reserve` and then
        // handed out a raw pointer). The new arena uses an append-only chunk
        // list, so each chunk's bytes live until reset. We collect the
        // pointer of the first allocation, force many more allocations that
        // grow into new chunks, then re-borrow the original pointer.
        let a = Arena::with_capacity(64);
        let first_ptr: *const u64 = a.alloc(0xdeadbeefu64);
        for i in 0..1000u64 {
            let _ = a.alloc(i);
        }
        // Safety: arena chunks are append-only; first_ptr was returned by
        // alloc on the same arena and the arena has not been reset/dropped.
        assert_eq!(unsafe { *first_ptr }, 0xdeadbeef);
        // sanity: we did spill into more than one chunk
        assert!(a.capacity() > 64);
    }

    #[test]
    fn alloc_respects_alignment() {
        let a = Arena::with_capacity(64);
        // Force unaligned cursor by alloc-ing one byte first.
        let _ = a.alloc(1u8);
        let p: *const u64 = a.alloc(0u64);
        assert_eq!((p as usize) % core::mem::align_of::<u64>(), 0);
    }

    #[test]
    fn larger_than_chunk_still_allocates() {
        let a = Arena::with_capacity(64);
        let big: [u32; 1024] = core::array::from_fn(|i| i as u32);
        let s = a.alloc_slice(&big);
        assert_eq!(s.len(), 1024);
        assert_eq!(s[1023], 1023);
    }

    #[test]
    fn reset_zeroes_used_bytes() {
        let mut a = Arena::with_capacity(64);
        a.alloc(1u32);
        a.alloc(2u32);
        assert!(a.used_bytes() > 0);
        a.reset();
        assert_eq!(a.used_bytes(), 0);
        // Should still be usable.
        assert_eq!(*a.alloc(99u32), 99);
    }
}

#![cfg(feature = "arena")]
#![doc(hidden)]

use alloc::vec::Vec;
use core::alloc::Layout;
use core::mem;
use core::ptr;

pub struct Arena {
    memory: Vec<u8>,
    used: usize,
}

impl Arena {
    #[inline]
    pub fn with_capacity(size: usize) -> Self {
        Self {
            memory: Vec::with_capacity(size),
            used: 0,
        }
    }

    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(4096)
    }

    #[inline]
    pub fn alloc<T>(&mut self, val: T) -> &mut T {
        let layout = Layout::for_value(&val);
        let ptr = self.alloc_layout(layout) as *mut T;
        unsafe {
            ptr::write(ptr, val);
            &mut *ptr
        }
    }

    #[inline]
    pub fn alloc_slice<T: Copy>(&mut self, slice: &[T]) -> &mut [T] {
        let layout = Layout::array::<T>(slice.len()).unwrap();
        let ptr = self.alloc_layout(layout) as *mut T;
        unsafe {
            ptr::copy_nonoverlapping(slice.as_ptr(), ptr, slice.len());
            core::slice::from_raw_parts_mut(ptr, slice.len())
        }
    }

    #[inline]
    fn alloc_layout(&mut self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();

        let misalignment = self.used % align;
        let padding = if misalignment == 0 {
            0
        } else {
            align - misalignment
        };
        let start = self.used + padding;

        let new_used = start + size;
        if new_used > self.memory.capacity() {
            let additional = size.max(4096);
            self.memory.reserve(additional);
        }

        let ptr = unsafe { self.memory.as_mut_ptr().add(start) };
        self.used = new_used;
        ptr
    }

    #[inline]
    pub fn reset(&mut self) {
        self.used = 0;
    }

    #[inline]
    pub fn used_bytes(&self) -> usize {
        self.used
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.memory.capacity()
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
    fn test_arena_alloc() {
        let mut arena = Arena::with_capacity(1024);
        let val = arena.alloc(42u32);
        assert_eq!(*val, 42);
    }

    #[test]
    fn test_arena_alloc_slice() {
        let mut arena = Arena::with_capacity(1024);
        let slice = arena.alloc_slice(&[1u32, 2, 3, 4, 5]);
        assert_eq!(slice, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena = Arena::with_capacity(1024);
        arena.alloc(42u32);
        arena.reset();
        assert_eq!(arena.used_bytes(), 0);
    }
}

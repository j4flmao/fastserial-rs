import os
import re

def fix_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    if 'io_buffer_tests.rs' in filepath:
        content = content.replace('ReadBuffer::new(data)', 'ReadBuffer::new(&data)')

    if 'simd_boundary.rs' in filepath:
        # replace &mut input.into_bytes() with &mut input.clone().into_bytes()? No, it drops.
        # Just need to bind it.
        # Actually it's easier to just find the function bodies and inject let mut input_vec = input.into_bytes();
        content = content.replace('decode(&mut input.into_bytes(), &_arena)', 'decode(&mut { let mut v = input.into_bytes(); v }, &_arena)')
        # Wait, block expression also drops the value! 
        pass

    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

for root, _, files in os.walk('d:/fastserial-rs'):
    if 'target' in root: continue
    for file in files:
        if file.endswith('.rs'):
            fix_file(os.path.join(root, file))

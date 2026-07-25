import os
import re

for root, _, files in os.walk('d:/fastserial-rs/tests'):
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        # Add let _arena = Arena::new(); to start of functions
        content = re.sub(r'(fn\s+[a-zA-Z0-9_]+\s*\([^)]*\)\s*(?:->\s*[^\{]+)?\{)', r'\1\n    let _arena = fastserial::arena::Arena::new();', content)
        
        # Change immutable variables to mutable vecs
        content = re.sub(r'let input = (br?#?".*?"#?);', r'let mut input = \1.to_vec();', content)
        content = re.sub(r'let bytes: &\[u8\] = (br?#?".*?"#?);', r'let mut bytes: Vec<u8> = \1.to_vec();', content)
        content = re.sub(r'let bytes = (br?#?".*?"#?);', r'let mut bytes = \1.to_vec();', content)
        content = re.sub(r'let data: &\[u8\] = (br?#?".*?"#?);', r'let mut data: Vec<u8> = \1.to_vec();', content)
        content = re.sub(r'let data = (br?#?".*?"#?);', r'let mut data = \1.to_vec();', content)
        
        # Add mut to variables returned from encode
        content = re.sub(r'let encoded =', 'let mut encoded =', content)
        content = re.sub(r'let json =', 'let mut json =', content)
        
        # Fix decode arguments
        content = re.sub(r'decode\((.*?)\)', r'decode(\1, &_arena)', content)
        content = re.sub(r'decode_raw\((.*?)\)', r'decode_raw(\1, &_arena)', content)
        content = re.sub(r'decode_str\((.*?)\)', r'decode_str(\1, &_arena)', content)
        
        # Fix & to &mut for the common buffers
        content = re.sub(r'decode\(&encoded,', 'decode(&mut encoded,', content)
        content = re.sub(r'decode\(&json,', 'decode(&mut json,', content)
        content = re.sub(r'decode\(&bytes,', 'decode(&mut bytes,', content)
        content = re.sub(r'decode\(&data,', 'decode(&mut data,', content)
        content = re.sub(r'decode\(&input,', 'decode(&mut input,', content)
        content = re.sub(r'decode\(input,', 'decode(&mut input,', content)
        content = re.sub(r'decode\(b"(.*?)",', r'decode(&mut b"\1".to_vec(),', content)
        
        content = re.sub(r'decode_raw\(&encoded,', 'decode_raw(&mut encoded,', content)
        content = re.sub(r'decode_raw\(&json,', 'decode_raw(&mut json,', content)
        content = re.sub(r'decode_raw\(&bytes,', 'decode_raw(&mut bytes,', content)
        content = re.sub(r'decode_raw\(&data,', 'decode_raw(&mut data,', content)
        content = re.sub(r'decode_raw\(&input,', 'decode_raw(&mut input,', content)
        content = re.sub(r'decode_raw\(input,', 'decode_raw(&mut input,', content)
        
        content = re.sub(r'decode_str\(&s,', 'decode_str(&mut s,', content)
        content = re.sub(r'decode_str\(&mut s,', 'decode_str(&mut s,', content) # deduplicate
        content = re.sub(r'let s =', 'let mut s =', content)

        # Handle ReadBuffer new
        content = re.sub(r'ReadBuffer::new\(&mut data\)', 'ReadBuffer::new(&mut data)', content) # unchanged basically
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

# Do the same for benches and examples just in case
for root, _, files in os.walk('d:/fastserial-rs/benches'):
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        content = re.sub(r'(fn\s+[a-zA-Z0-9_]+\s*\([^)]*\)\s*(?:->\s*[^\{]+)?\{)', r'\1\n    let _arena = fastserial::arena::Arena::new();', content)
        content = re.sub(r'decode\((.*?)\)', r'decode(\1, &_arena)', content)
        content = re.sub(r'decode_raw\((.*?)\)', r'decode_raw(\1, &_arena)', content)
        content = re.sub(r'decode\(&encoded,', 'decode(&mut encoded,', content)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

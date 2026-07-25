import os
import re

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    # Find all decode calls
    # We want to replace decode(X) with decode(X, &arena)
    # And we want to ensure let arena = fastserial::arena::Arena::new(); is in the function
    
    # Simple approach: just inject let arena = fastserial::arena::Arena::new(); after every { of a function that contains decode
    # But it's easier to just use &fastserial::arena::Arena::new() and fix the borrow checker errors manually? No, there are 50+ errors.
    
    # Better approach: 
    # Just replace decode(X) with decode(X, &fastserial::arena::Arena::new()) first, 
    # then let the compiler tell us which ones need a local rena binding, and we fix those manually.
    
    new_content = re.sub(r'decode\((.*?)(?<!arena)\)', r'decode(\1, &fastserial::arena::Arena::new())', content)
    new_content = re.sub(r'decode_str\((.*?)(?<!arena)\)', r'decode_str(\1, &fastserial::arena::Arena::new())', new_content)
    new_content = re.sub(r'decode_raw\((.*?)(?<!arena)\)', r'decode_raw(\1, &fastserial::arena::Arena::new())', new_content)
    
    # Clean up double injections if any
    new_content = new_content.replace(', &fastserial::arena::Arena::new(), &fastserial::arena::Arena::new()', ', &fastserial::arena::Arena::new()')
    
    if content != new_content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)

for root, _, files in os.walk('d:/fastserial-rs/tests'):
    for file in files:
        if file.endswith('.rs'):
            process_file(os.path.join(root, file))

for root, _, files in os.walk('d:/fastserial-rs/benches'):
    for file in files:
        if file.endswith('.rs'):
            process_file(os.path.join(root, file))
            
for root, _, files in os.walk('d:/fastserial-rs/examples'):
    for file in files:
        if file.endswith('.rs'):
            process_file(os.path.join(root, file))

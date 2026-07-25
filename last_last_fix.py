import os
import re

for root, _, files in os.walk('d:/fastserial-rs'):
    if 'target' in root or '.git' in root or 'api-bench' in root or 'sample-axum' in root:
        continue
    if not ('tests' in root or 'examples' in root or 'benches' in root):
        continue
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        content = content.replace('input.into_bytes(, &_arena)', 'input.into_bytes()')
        
        content = re.sub(r'let mut (input|json|data|encoded|bytes) = (br?#?".*?"#?);', r'let mut \1 = \2.to_vec();', content)
        
        content = re.sub(r'decode\(&mut\s+(br?#?".*?"#?),\s*&_arena\)', r'decode(&mut \1.to_vec(), &_arena)', content)
        
        content = content.replace('json::decode(&json_bytes)', 'json::decode(&mut json_bytes.to_vec(), &fastserial::arena::Arena::new())')
        
        content = content.replace('binary_decode(data)', 'binary_decode(&mut data, &_arena)')
        
        content = content.replace('scan_quote_or_backslash(input)', 'scan_quote_or_backslash(&input)')

        # Fix any decode(&input) -> decode(&mut input, &_arena)
        content = re.sub(r'decode\(&input\)', r'decode(&mut input, &_arena)', content)

        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

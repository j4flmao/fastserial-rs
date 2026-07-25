import os
import re

for root, _, files in os.walk('d:/fastserial-rs/tests'):
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        content = re.sub(r'(fn\s+[a-zA-Z0-9_]+\s*\([^)]*\)\s*(?:->\s*[^\{]+)?\s*\{\s*)', r'\1let _arena = fastserial::arena::Arena::new();\n    ', content)
        content = re.sub(r'let\s+(?:mut\s+)?([a-zA-Z0-9_]+)\s*(?::\s*&\[u8\])?\s*=\s*(br?#?".*?"#?)\s*;', r'let mut \1 = \2.to_vec();', content)
        content = re.sub(r'let\s+(?!mut)([a-zA-Z0-9_]+)\s*=\s*vec!', r'let mut \1 = vec!', content)
        content = re.sub(r'let\s+(?!mut)([a-zA-Z0-9_]+)\s*=\s*encode\(', r'let mut \1 = encode(', content)
        
        def fix_decode(match):
            func = match.group(1) 
            arg = match.group(2)
            if re.match(r'^\s*br?#?".*?"#?\s*$', arg):
                return f"{func}({arg}, &_arena)" 
            arg = arg.strip()
            if arg.startswith('&mut '):
                new_arg = arg
            elif arg.startswith('&'):
                new_arg = '&mut ' + arg[1:]
            else:
                new_arg = '&mut ' + arg
            return f"{func}({new_arg}, &_arena)"
            
        content = re.sub(r'\b(decode|decode_raw|decode_str)\s*\(\s*(.*?)\s*\)', fix_decode, content)
        
        def fix_inline_literal(match):
            prefix = match.group(1) 
            func = match.group(2) 
            lit = match.group(3) 
            suffix = match.group(4) 
            return f"let mut _buf = {lit}.to_vec(); {prefix}{func}(&mut _buf{suffix};"
            
        content = re.sub(r'(let\s+[^=]+\s*=\s*)(decode|decode_raw|decode_str)\s*\(\s*(br?#?".*?"#?)\s*(.*?);', fix_inline_literal, content)
        
        def fix_inline_literal_assert(match):
            prefix = match.group(1) 
            func = match.group(2)
            lit = match.group(3)
            suffix = match.group(4)
            return f"{{ let mut _buf = {lit}.to_vec(); {prefix}{func}(&mut _buf{suffix}; }}"
            
        content = re.sub(r'(assert_eq!\s*\(\s*)(decode|decode_raw|decode_str)\s*\(\s*(br?#?".*?"#?)\s*(.*?);', fix_inline_literal_assert, content)
        content = re.sub(r'\b(decode|decode_raw|decode_str)\s*\(\s*(br?#?".*?"#?)\s*(,\s*&_arena\))', r'\1(&mut \2.to_vec()\3', content)
        content = re.sub(r'let\s+s\s*=', 'let mut s =', content)
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

for root, _, files in os.walk('d:/fastserial-rs/benches'):
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        content = re.sub(r'(fn\s+[a-zA-Z0-9_]+\s*\([^)]*\)\s*(?:->\s*[^\{]+)?\s*\{\s*)', r'\1let _arena = fastserial::arena::Arena::new();\n    ', content)
        content = re.sub(r'\b(decode|decode_raw)\s*\(\s*&mut\s+(.*?)\s*\)', r'\1(&mut \2, &_arena)', content)
        content = re.sub(r'\b(decode|decode_raw)\s*\(\s*&(?!mut)(.*?)\s*\)', r'\1(&mut \2, &_arena)', content)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

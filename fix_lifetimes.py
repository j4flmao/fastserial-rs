import os
import re

for root, _, files in os.walk('d:/fastserial-rs/tests'):
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        # Fix: let x = decode(&mut y.to_vec(), &_arena);
        # Wait, there can be multiple on the same line or over multiple lines, but mostly single line
        def repl(match):
            prefix = match.group(1) # e.g. "let s: &str = "
            buf_expr = match.group(2) # e.g. "b\"hello\".to_vec()"
            suffix = match.group(3) # e.g. ".unwrap();"
            return f"let mut _buf = {buf_expr}; {prefix}decode(&mut _buf, &_arena){suffix}"

        content = re.sub(r'(let\s+[^=]+\s*=\s*)decode\(&mut\s+(.*?\.to_vec\(\)),\s*&_arena\)(.*?);', repl, content)
        content = re.sub(r'(let\s+[^=]+\s*=\s*)decode_raw\(&mut\s+(.*?\.to_vec\(\)),\s*&_arena\)(.*?);', repl, content)
        content = re.sub(r'(let\s+[^=]+\s*=\s*)decode_str\(&mut\s+(.*?\.to_vec\(\)),\s*&_arena\)(.*?);', repl, content)

        # What if it's assert_eq!(decode(&mut b"...".to_vec(), &_arena), ...)?
        def repl_assert(match):
            prefix = match.group(1)
            buf_expr = match.group(2)
            suffix = match.group(3)
            return f"{{ let mut _buf = {buf_expr}; {prefix}decode(&mut _buf, &_arena){suffix} }}"
            
        content = re.sub(r'(assert_eq!\(\s*)decode\(&mut\s+(.*?\.to_vec\(\)),\s*&_arena\)(.*?);', repl_assert, content)
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

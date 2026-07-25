import os
import re

for root, _, files in os.walk('d:/fastserial-rs/tests'):
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        # Fix function signatures to include let _arena
        content = re.sub(r'(?m)^(fn\s+[a-zA-Z0-9_]+\s*\([^)]*\)\s*(?:->\s*[^\{]+)?\s*\{\s*)(?!let _arena)', r'\1let _arena = fastserial::arena::Arena::new();\n    ', content)
        
        # We find every decode/decode_raw/decode_str call.
        def fix_decode_call(match):
            func_name = match.group(1)
            arg = match.group(2).strip()
            
            # If it already has _arena, we skip
            if '_arena' in arg:
                return match.group(0)
                
            # If arg is &var -> &mut var
            if arg.startswith('&') and not arg.startswith('&mut '):
                arg = '&mut ' + arg[1:]
            elif not arg.startswith('&mut '):
                arg = '&mut ' + arg
                
            return f"{func_name}({arg}, &_arena)"
            
        content = re.sub(r'\b(decode|decode_raw|decode_str)\s*\(\s*([^,]+?)\s*\)', fix_decode_call, content)
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

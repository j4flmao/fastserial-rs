import os
import re

for root, _, files in os.walk('d:/fastserial-rs/tests'):
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        content = re.sub(r'(fn\s+[a-zA-Z0-9_]+\s*\([^)]*\)\s*(?:->\s*[^\{]+)?\{)', r'\1\n    let _arena = fastserial::arena::Arena::new();', content)
        
        content = re.sub(r'\blet bytes: &\[u8\] =', 'let mut bytes: Vec<u8> =', content)
        content = re.sub(r'\blet bytes =', 'let mut bytes =', content)
        content = re.sub(r'\blet data: &\[u8\] =', 'let mut data: Vec<u8> =', content)
        content = re.sub(r'\blet data =', 'let mut data =', content)
        content = re.sub(r'\blet input =', 'let mut input =', content)
        content = re.sub(r'\blet encoded =', 'let mut encoded =', content)
        content = re.sub(r'\blet json =', 'let mut json =', content)
        content = re.sub(r'\blet s =', 'let mut s =', content)
        
        content = re.sub(r'let mut bytes(.*?)= (b".*?");', r'let mut bytes\1= \2.to_vec();', content)
        content = re.sub(r'let mut data(.*?)= (b".*?");', r'let mut data\1= \2.to_vec();', content)
        content = re.sub(r'let mut input(.*?)= (b".*?");', r'let mut input\1= \2.to_vec();', content)
        content = re.sub(r'let mut input(.*?)= (br#".*?"#);', r'let mut input\1= \2.to_vec();', content)
        
        def fix_args(match):
            func = match.group(1) 
            args = match.group(2)
            args = re.sub(r'&encoded\b', '&mut encoded', args)
            args = re.sub(r'&json\b', '&mut json', args)
            args = re.sub(r'&bytes\b', '&mut bytes', args)
            args = re.sub(r'&data\b', '&mut data', args)
            args = re.sub(r'&input\b', '&mut input', args)
            args = re.sub(r'&s\b', '&mut s', args)
            
            if args == 'input' or args.startswith('input,'):
                args = re.sub(r'^input\b', '&mut input', args)
                
            if re.match(r'^br?#?".*?"#?', args):
                args = re.sub(r'^(br?#?".*?"#?)', r'__TMP__\1', args)
                
            return f"{func}({args}, &_arena)"
            
        content = re.sub(r'\b(decode|decode_raw|decode_str)\((.*?)\)', fix_args, content)
        
        def fix_tmp(match):
            prefix = match.group(1) 
            func = match.group(2) 
            string_lit = match.group(3) 
            suffix = match.group(4) 
            return f"let mut _buf = {string_lit}.to_vec(); {prefix}{func}(&mut _buf{suffix};"
            
        content = re.sub(r'(.*?)\b(decode|decode_raw|decode_str)\(__TMP__(br?#?".*?"#?)(.*?);', fix_tmp, content)
        
        def fix_tmp_assert(match):
            prefix = match.group(1) 
            func = match.group(2)
            string_lit = match.group(3)
            suffix = match.group(4)
            return f"{{ let mut _buf = {string_lit}.to_vec(); {prefix}{func}(&mut _buf{suffix}; }}"
            
        content = re.sub(r'(assert_eq!\(.*?)\b(decode|decode_raw|decode_str)\(__TMP__(br?#?".*?"#?)(.*?);', fix_tmp_assert, content)
        
        content = re.sub(r'__TMP__(br?#?".*?"#?)', r'&mut \1.to_vec()', content)
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

for root, _, files in os.walk('d:/fastserial-rs/benches'):
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        content = re.sub(r'(fn\s+[a-zA-Z0-9_]+\s*\([^)]*\)\s*(?:->\s*[^\{]+)?\{)', r'\1\n    let _arena = fastserial::arena::Arena::new();', content)
        content = re.sub(r'\b(decode|decode_raw)\((.*?)\)', r'\1(\2, &_arena)', content)
        content = re.sub(r'\b(decode|decode_raw)\(&encoded,', r'\1(&mut encoded,', content)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

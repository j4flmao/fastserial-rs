import os
import re

for filepath in ['d:/fastserial-rs/tests/derive_struct_tests.rs', 'd:/fastserial-rs/tests/json_codec_tests.rs']:
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    # fix missing mut on json
    content = re.sub(r'let json\s*=', 'let mut json =', content)
    
    # fix missing to_vec() on byte literals assigned to mut json
    content = re.sub(r'let mut json(.*?)=\s*(br?#?".*?"#?)\s*;', r'let mut json\1= \2.to_vec();', content)
    
    # fix E0308 when passing &mut json where json is a byte literal (missed above)
    content = re.sub(r'decode\(&mut\s+(b".*?"),\s*&_arena\)', r'decode(&mut \1.to_vec(), &_arena)', content)

    # ensure that b"..." passed directly is correctly converted in case it wasn't
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

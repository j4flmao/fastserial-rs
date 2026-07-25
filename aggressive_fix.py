import os
import re

for root, _, files in os.walk('d:/fastserial-rs/tests'):
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        # let bytes = -> let mut bytes =
        content = re.sub(r'\blet bytes\b', 'let mut bytes', content)
        content = re.sub(r'\blet json\b', 'let mut json', content)
        content = re.sub(r'\blet encoded\b', 'let mut encoded', content)
        content = re.sub(r'\blet data\b', 'let mut data', content)
        content = re.sub(r'\blet input\b', 'let mut input', content)
        content = re.sub(r'\blet s =', 'let mut s =', content)
        
        # &bytes -> &mut bytes
        content = re.sub(r'decode\(&bytes,', 'decode(&mut bytes,', content)
        content = re.sub(r'decode\(&json,', 'decode(&mut json,', content)
        content = re.sub(r'decode\(&encoded,', 'decode(&mut encoded,', content)
        content = re.sub(r'decode\(&data,', 'decode(&mut data,', content)
        content = re.sub(r'decode\(&input,', 'decode(&mut input,', content)
        
        content = re.sub(r'decode_raw\(&bytes,', 'decode_raw(&mut bytes,', content)
        content = re.sub(r'decode_raw\(&json,', 'decode_raw(&mut json,', content)
        content = re.sub(r'decode_raw\(&encoded,', 'decode_raw(&mut encoded,', content)
        content = re.sub(r'decode_raw\(&data,', 'decode_raw(&mut data,', content)
        content = re.sub(r'decode_raw\(&input,', 'decode_raw(&mut input,', content)
        
        # For arrays like decode(input, ...) or decode(b"...", ...)
        content = re.sub(r'decode\(\s*(b"[^"]*")\s*,', r'decode(&mut \1.to_vec(),', content)
        content = re.sub(r'decode\(\s*(b\s*r#"[^"]*"#)\s*,', r'decode(&mut \1.to_vec(),', content)
        
        # for decode(input, ...) where input is a slice or array
        content = re.sub(r'decode\(input,', 'decode(&mut input.to_vec(),', content)
        
        # fastserial::decode(&mut vec, ...) vs fastserial::decode(&mut slice, ...)
        # some test still use decode(json.as_mut_slice(), ...) which is fine.
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

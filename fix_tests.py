import os
import json
import subprocess
import re

def run_check():
    result = subprocess.run(['cargo', 'check', '--tests', '--message-format=json'], capture_output=True, text=True)
    return result.stdout.splitlines()

def fix_errors():
    lines = run_check()
    fixed = 0
    for line in lines:
        if not line.strip(): continue
        try:
            msg = json.loads(line)
        except:
            continue
        
        if msg.get('reason') != 'compiler-message':
            continue
            
        message = msg['message']
        code = message.get('code')
        if not code: continue
        
        code_id = code.get('code')
        spans = message.get('spans', [])
        
        for span in spans:
            if not span.get('is_primary'): continue
            file_name = span['file_name']
            line_start = span['line_start']
            
            with open(file_name, 'r', encoding='utf-8') as f:
                content_lines = f.readlines()
                
            idx = line_start - 1
            if idx < 0 or idx >= len(content_lines): continue
            
            line_text = content_lines[idx]
            
            if code_id == 'E0308':
                if 'found reference &[' in message['message']:
                    # array reference, need to clone or convert to vec
                    content_lines[idx] = re.sub(r'decode\(b"(.*?)",', r'decode(&mut b"\1".to_vec(),', line_text)
                    content_lines[idx] = re.sub(r'decode\((input.*?),', r'decode(&mut \1.to_vec(),', content_lines[idx])
                    content_lines[idx] = re.sub(r'decode_raw\((input.*?),', r'decode_raw(&mut \1.to_vec(),', content_lines[idx])
                    content_lines[idx] = re.sub(r'decode_str\((input.*?),', r'decode_str(&mut \1.to_vec(),', content_lines[idx])
                    
                elif 'found reference &std::vec::Vec' in message['message'] or 'found reference &alloc::vec::Vec' in message['message']:
                    content_lines[idx] = line_text.replace('&encoded', '&mut encoded')
                    content_lines[idx] = content_lines[idx].replace('&json', '&mut json')
                    content_lines[idx] = content_lines[idx].replace('&bytes', '&mut bytes')
                    content_lines[idx] = content_lines[idx].replace('&data', '&mut data')
                    
                fixed += 1
            elif code_id == 'E0596':
                content_lines[idx] = line_text.replace('let data', 'let mut data').replace('let input', 'let mut input').replace('let buf', 'let mut buf').replace('let encoded', 'let mut encoded')
                if idx > 0:
                    content_lines[idx-1] = content_lines[idx-1].replace('let data', 'let mut data').replace('let input', 'let mut input').replace('let encoded', 'let mut encoded').replace('let json', 'let mut json').replace('let bytes', 'let mut bytes')
                fixed += 1
                
            with open(file_name, 'w', encoding='utf-8') as f:
                f.writelines(content_lines)
    return fixed

for _ in range(5):
    if fix_errors() == 0:
        break

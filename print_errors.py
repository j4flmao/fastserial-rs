import os
import json
import subprocess

result = subprocess.run(['cargo', 'check', '--tests', '--message-format=json'], capture_output=True, text=True)
lines = result.stdout.splitlines()

errors = []
for line in lines:
    if not line.strip(): continue
    try:
        msg = json.loads(line)
        if msg.get('reason') != 'compiler-message': continue
        message = msg['message']
        code = message.get('code')
        if not code: continue
        for span in message.get('spans', []):
            if span.get('is_primary'):
                errors.append(f"{span['file_name']}:{span['line_start']} - {code['code']}")
    except:
        pass

for e in sorted(set(errors)):
    print(e)

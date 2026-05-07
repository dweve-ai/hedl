import tomllib

with open('crates/hedl-filter/filters/make.toml', 'rb') as f:
    data = tomllib.load(f)

test = data['tests']['make'][0]
input_val = test['input']

s = input_val
print('Has newline:', '\n' in s)
print('Has comma:', ',' in s)
print('Has quote:', '"' in s)

if ',' in s or '\n' in s or '"' in s:
    s = s.replace('\\', '\\\\')
    s = s.replace('\n', '\\n')
    s = s.replace('\t', '\\t')
    s = s.replace('"', '""')
    print('Escaped:', repr(s[:100]))
else:
    print('Not escaped')

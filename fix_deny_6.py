with open("deny.toml", "r") as f:
    lines = f.read().split('\n')

for i, line in enumerate(lines):
    if '"siphasher",' in line:
        lines.insert(i, '    "signature",')
        break

with open("deny.toml", "w") as f:
    f.write('\n'.join(lines))

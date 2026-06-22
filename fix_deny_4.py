with open("deny.toml", "r") as f:
    lines = f.read().split('\n')

for i, line in enumerate(lines):
    if '"solana-instruction",' in line:
        # We need to insert solana-account-info and siphasher too
        lines.insert(i, '    "solana-account-info",\n    "siphasher",')
        break

for i, line in enumerate(lines):
    if '"solana-invoke",' in line:
        lines.pop(i)
        break

with open("deny.toml", "w") as f:
    f.write('\n'.join(lines))

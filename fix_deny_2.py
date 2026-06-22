with open("deny.toml", "r") as f:
    lines = f.read().split('\n')

for i, line in enumerate(lines):
    if '"solana-slot-history",' in line:
        # Need to insert solana-instructions-sysvar as well as the rest
        lines.insert(i, '    "solana-instructions-sysvar",')
        break

with open("deny.toml", "w") as f:
    f.write('\n'.join(lines))

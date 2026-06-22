with open("deny.toml", "r") as f:
    lines = f.read().split('\n')

for i, line in enumerate(lines):
    if '"twox-hash",' in line:
        lines[i] = line + '\n    "solana-slot-history",\n    "solana-stable-layout",\n    "solana-stake-interface",\n    "solana-system-interface",\n    "solana-sysvar",\n    "solana-sysvar-id",\n    "spki",\n    "strum",\n    "supports-color",\n    "synstructure",\n    "system-configuration",\n    "system-configuration-sys",\n    "tower-http",\n    "uint",\n    "uuid",\n    "wasi",'
        break

with open("deny.toml", "w") as f:
    f.write('\n'.join(lines))

with open("deny.toml", "r") as f:
    lines = f.read().split('\n')

for i, line in enumerate(lines):
    if '"solana-slot-history",' in line:
        lines.insert(i, '    "solana-instruction",\n    "solana-cpi",\n    "solana-invoke",\n    "solana-address",\n    "solana-clock",\n    "solana-epoch-rewards",\n    "solana-epoch-schedule",\n    "solana-feature-gate-interface",\n    "solana-fee-calculator",\n    "solana-hash",\n    "solana-atomic-u64",\n    "solana-define-syscall",')
        break

with open("deny.toml", "w") as f:
    f.write('\n'.join(lines))

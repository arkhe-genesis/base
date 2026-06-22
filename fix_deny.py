with open("deny.toml", "r") as f:
    lines = f.read().split('\n')

for i, line in enumerate(lines):
    if '"solana-slot-history",' in line:
        # We need to insert solana-slot-hashes, solana-msg, solana-program-entrypoint, solana-program-error, solana-program-memory, solana-program-option, solana-program-pack, solana-pubkey, solana-rent, solana-sanitize, solana-sdk-ids, solana-sdk-macro, solana-serialize-utils, solana-sha256-hasher, solana-slot-hashes
        lines.insert(i, '    "solana-loader-v3-interface",\n    "solana-msg",\n    "solana-program-entrypoint",\n    "solana-program-error",\n    "solana-program-memory",\n    "solana-program-option",\n    "solana-program-pack",\n    "solana-pubkey",\n    "solana-rent",\n    "solana-sanitize",\n    "solana-sdk-ids",\n    "solana-sdk-macro",\n    "solana-serialize-utils",\n    "solana-sha256-hasher",\n    "solana-slot-hashes",\n    "solana-last-restart-slot",')
        break

with open("deny.toml", "w") as f:
    f.write('\n'.join(lines))

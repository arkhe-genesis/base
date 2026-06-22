with open("deny.toml", "r") as f:
    lines = f.read().split('\n')

for i, line in enumerate(lines):
    if '"synstructure",' in line:
        lines.insert(i, '    "ark-bn254",\n    "ark-ec",\n    "ark-ff",\n    "ark-ff-asm",\n    "ark-ff-macros",\n    "ark-poly",\n    "ark-serialize",\n    "ark-serialize-derive",\n    "ark-std",\n    "asn1-rs",\n    "asn1-rs-derive",\n    "asn1-rs-impl",\n    "async-channel",\n    "bech32",\n    "bitcoin",\n    "bitcoin_hashes",\n    "borsh",\n    "borsh-derive",\n    "coins-bip32",\n    "coins-bip39",\n    "coins-core",\n    "curve25519-dalek",\n    "dashmap",\n    "der",\n    "der-parser",\n    "ed25519",\n    "ed25519-dalek",\n    "enr",\n    "event-listener",\n    "five8",\n    "five8_const",\n    "five8_core",\n    "fixed-hash",\n    "gethostname",\n    "governor",\n    "heck",\n    "hermit-abi",\n    "hex-conservative",\n    "jsonwebtoken",\n    "linux-raw-sys",\n    "memmap2",\n    "ndarray",\n    "num",\n    "num-complex",\n    "num-rational",\n    "oid-registry",\n    "pbkdf2",\n    "pem",\n    "pkcs8",\n    "primitive-types",\n    "procfs",\n    "procfs-core",\n    "redox_syscall",\n    "reqwest-middleware",\n    "ring",\n    "rustix",\n    "rustls-pemfile",\n    "schemars",\n    "schemars_derive",\n    "scrypt",\n    "shlex",')
        break

with open("deny.toml", "w") as f:
    f.write('\n'.join(lines))

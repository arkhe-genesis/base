#!/bin/bash
cd crates/utilities/test-utils/contracts
rm -rf dependencies
mkdir dependencies
git clone https://github.com/foundry-rs/forge-std.git dependencies/forge-std-1.14.0
cd dependencies/forge-std-1.14.0
git checkout v1.14.0
cd ../..
git clone https://github.com/OpenZeppelin/openzeppelin-contracts.git dependencies/openzeppelin-contracts-5.5.0
cd dependencies/openzeppelin-contracts-5.5.0
git checkout v5.5.0
cd ../..
git clone https://github.com/transmissions11/solmate.git dependencies/solmate-6.8.0
cd dependencies/solmate-6.8.0
git checkout 89365b880c4f3c786bdd453d4b8e8fe410344a69
cd ../..
cat << 'TOML' > foundry.toml
[profile.default]
src = "src"
out = "out"
libs = ["dependencies"]
remappings = [
    "forge-std/=dependencies/forge-std-1.14.0/src/",
    "@openzeppelin/=dependencies/openzeppelin-contracts-5.5.0/",
    "solmate/=dependencies/solmate-6.8.0/src/",
    "ds-test/=dependencies/solmate-6.8.0/lib/ds-test/src/",
    "erc4626-tests/=dependencies/openzeppelin-contracts-5.5.0/lib/erc4626-tests/",
    "halmos-cheatcodes/=dependencies/openzeppelin-contracts-5.5.0/lib/halmos-cheatcodes/src/",
    "openzeppelin-contracts/=dependencies/openzeppelin-contracts-5.5.0/"
]

[dependencies]
# forge-std = "1.14.0"
# openzeppelin-contracts = { version = "5.5.0", git = "https://github.com/OpenZeppelin/openzeppelin-contracts.git", tag = "v5.5.0" }
# solmate = { version = "6.8.0", git = "https://github.com/transmissions11/solmate.git", rev = "89365b880c4f3c786bdd453d4b8e8fe410344a69" }
TOML

~/.foundry/bin/forge build

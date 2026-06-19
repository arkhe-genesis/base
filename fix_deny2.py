import toml

with open("deny.toml", "r") as f:
    config = toml.load(f)

# Change multiple-versions = "deny" to "warn"
if "bans" in config:
    config["bans"]["multiple-versions"] = "warn"
    if "skip" in config["bans"]:
        del config["bans"]["skip"]

with open("deny.toml", "w") as f:
    toml.dump(config, f)

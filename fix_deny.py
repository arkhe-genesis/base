import tomli
import tomli_w

with open('deny.toml', 'rb') as f:
    config = tomli.load(f)

if 'bans' in config and 'skip' in config['bans']:
    skips = config['bans']['skip']

    # We want to keep x509-parser or whatever is needed
    # But wait, we can just remove all unused skips and add what's missing
    pass

Docker Container for a slightly more robust Palworld Server that allows IPv6 connectivity and easy configuration with a toml file.

Make sure you mount `/config` (your folder with the toml config as `palworld_conf.toml` in it) and `/home/steam/palworld/Pal/Saved/SaveGames` (No need to put anything here, this is to persist your world and player data).
Map Port 8211 UDP (plus RCON and REST if you specify in settings) to the host.

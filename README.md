Custom Palworld server wrapper that
- allows better configuration with a toml file
- shuts down when nobody is connected and starts up again when someone wants to join

Mount:
* `/config`: folder with the toml config as `palworld_conf.toml` in it, as well as an optional `init.sh` file that will be run at the start
* `/home/steam/palworld/Pal/Saved/SaveGames/0`: No need to put anything here, this is to persist world and player data

Ports:
* 8211 UDP
* (RCON, REST if specified)

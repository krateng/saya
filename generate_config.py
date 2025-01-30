import toml
import os

targetfile = os.path.join(os.environ["PALSERVERDIR"], "Pal", "Saved", "Config", "LinuxServer", "PalWorldSettings.ini")
targetfile2 = os.path.join(os.environ["PALSERVERDIR"], "Pal", "Saved", "Config", "LinuxServer", "GameUserSettings.ini")
worlddir = os.path.join(os.environ["PALSERVERDIR"], "Pal", "Saved", "SaveGames", "0")


settings = toml.load("/config/palworld_conf.toml")
	
result = """
[/Script/Pal.PalGameWorldSettings]
OptionSettings=("""

for key,val in settings['Server'].items():
	if isinstance(val, int):
		val = str(val)
	elif isinstance(val, str):
		assert not ('"' in val)
		val = '"' + val + '"'
	elif isinstance(val, bool):
		val = str(val)
	else:
		print(f"Value type not implemented yet: {val}")
		raise ValueError()
	
	result += f"{key}={val},"

result += ")\n"



with open(targetfile, "w") as fd:
	fd.write(result)



# now set the correct world
try:
	worlds = os.listdir(worlddir)
except:
	worlds = []
	
	
if len(worlds) == 1:
	world = worlds[0]
	print("World", world, "detected, setting configuration.")

elif len(worlds) == 0:
	world = None
	print("No previous world, this seems to be a first time setup!")

else:
	world = worlds[0]
	print("Multiple worlds! Picking one, is this what you want?")

if world:
	with open(targetfile2, "w") as fd:
		fd.write(f"[/Script/Pal.PalGameLocalSettings]\nDedicatedServerName={world}")



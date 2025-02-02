use toml::Table;
use std::fs;
use std::path::Path;

const WORLDSETTINGS_PATH: &str = "./Pal/Saved/Config/LinuxServer/PalWorldSettings.ini";
const USERSETTINGS_PATH: &str = "./Pal/Saved/Config/LinuxServer/GameUserSettings.ini";
const WORLDS_FOLDER: &str = "Pal/Saved/SaveGames/0";
const SAYA_SETTINGS_PATH: &str = "/config/palworld_conf.toml";

pub fn generate_settings() {
    println!("Creating world settings file...");
    let config_file_content = fs::read_to_string(SAYA_SETTINGS_PATH).unwrap();
    let table = config_file_content.parse::<Table>().unwrap();

    const ERROR_MSG: &str = "TOML file invalid!";
    let settings_iter = table.get("Server").expect(ERROR_MSG).as_table().expect(ERROR_MSG).iter();


    let settings_string: String = settings_iter
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>() // Collect into Vec<String>
        .join(","); // Join with commas

    let settings_file_content = format!("[/Script/Pal.PalGameWorldSettings]\nOptionSettings=({})\n", settings_string);

    fs::write(WORLDSETTINGS_PATH, settings_file_content).unwrap();
}

pub fn set_world() {
    println!("Creating user settings file...");
    let worlds = list_folders(WORLDS_FOLDER).unwrap();
    let active_world = match worlds.len() {
        0 => {
            println!("No saved worlds, this is a new server installation!");
            None
        },
        1 => {
            println!("Found world folder, creating user settings...");
            worlds.first()
        },
        _ => {
            println!("Found multiple worlds! Picking one...");
            worlds.first()
        }
    };

    if let Some(world) = active_world {
        let settings_file_content = format!("[/Script/Pal.PalGameLocalSettings]\nDedicatedServerName={}\n", world);
        fs::write(USERSETTINGS_PATH, settings_file_content).unwrap();
    }



}


fn list_folders<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<String>> {
    let mut folders = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                folders.push(name.to_string());
            }
        }
    }
    Ok(folders)
}
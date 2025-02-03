use toml::Table;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn generate_settings(saya_conf_file: &Path, palworld_worldsettings_file: &Path) {
    println!("Creating world settings file...");
    let config_file_content = fs::read_to_string(saya_conf_file).unwrap();
    let table = config_file_content.parse::<Table>().unwrap();

    const ERROR_MSG: &str = "TOML file invalid!";
    let settings_iter = table.get("Server").expect(ERROR_MSG).as_table().expect(ERROR_MSG).iter();


    let settings_string: String = settings_iter
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>() // Collect into Vec<String>
        .join(","); // Join with commas

    let settings_file_content = format!("[/Script/Pal.PalGameWorldSettings]\nOptionSettings=({})\n", settings_string);

    fs::write(palworld_worldsettings_file, settings_file_content).unwrap();
}

pub fn set_world(worlds_folder: &Path, palworld_usersettings_file: &Path) {
    println!("Creating user settings file...");
    let worlds = list_folders(worlds_folder).unwrap();
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
        fs::write(palworld_usersettings_file, settings_file_content).unwrap();
    }
}

pub fn init_script(scriptfile: &Path) -> bool {
    if !scriptfile.exists() || !scriptfile.is_file() {
        println!("Could not find init script {:?}", scriptfile);
        return false;
    }
    println!("\n\nRunning custom init script...\n\n");
    Command::new("bash").arg(scriptfile).spawn().expect("Init script failed!");
    return true;
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
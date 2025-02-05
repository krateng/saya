use toml::{Table, Value};
use std::fs;
use std::path::Path;
use std::process::Command;
use crate::ServerSetupCheck;

pub fn generate_settings(saya_conf_file: &Path, palworld_worldsettings_file: &Path) -> Vec<ServerSetupCheck> {
    //println!("Creating world settings file...");
    let config_file_content = match fs::read_to_string(saya_conf_file) {
        Ok(v) => { v }
        Err(_) => {
            return vec!(ServerSetupCheck::ERROR {
                message: format!("Failed to open config file {}. Make sure it exists and is mounted as readable!", saya_conf_file.to_str().unwrap()),
            });
        }
    };
    let table = match config_file_content.parse::<Table>() {
        Ok(v) => { v }
        Err(_) => {
            return vec!(ServerSetupCheck::ERROR {
                message: format!("Failed to parse config file {}", saya_conf_file.to_str().unwrap()),
            })
        }
    };
    let settings = match table.get("Server") {
        None => {
            return vec!(ServerSetupCheck::ERROR { message: format!("Config file {} is missing top level key 'Server'", saya_conf_file.to_str().unwrap()) })
        }
        Some(v) => { v }
    };
    let settings_iter = match settings.as_table() {
        None => {
            return vec!(ServerSetupCheck::ERROR { message: format!("Config file {} has the wrong value type for 'Server'", saya_conf_file.to_str().unwrap()) });
        }
        Some(v) => { v.iter() }
    };

    let settings_string: String = settings_iter
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>() // Collect into Vec<String>
        .join(","); // Join with commas

    let settings_file_content = format!("[/Script/Pal.PalGameWorldSettings]\nOptionSettings=({})\n", settings_string);

    match fs::write(palworld_worldsettings_file, settings_file_content) {
        Ok(_) => vec!(ServerSetupCheck::OK { message: format!("Successfully created {}", palworld_worldsettings_file.to_str().unwrap()) }),
        Err(_) => vec!(ServerSetupCheck::ERROR { message: format!("Could not write {}", palworld_worldsettings_file.to_str().unwrap()) }),
    }
}

pub fn set_world(worlds_folder: &Path, palworld_usersettings_file: &Path) -> Vec<ServerSetupCheck> {
    //println!("Creating user settings file...");
    let worlds = match list_folders(worlds_folder) {
        Ok(v) => { v }
        Err(_) => { return vec!(ServerSetupCheck::ERROR { message: format!("Could not access world save folder: {}", worlds_folder.to_str().unwrap()) } ); }
    };

    let mut result = vec!();
    let active_world: Option<&String> = match worlds.len() {
        0 => {
            // New installation
            None
        },
        1 => {
            worlds.first()
        },
        _ => {
            let world = worlds.first();
            result.push(ServerSetupCheck::WARNING { message: format!("Multiple saved worlds were found, using {}!", world.unwrap()) });
            world
        }
    };

    if let Some(world) = active_world {
        let settings_file_content = format!("[/Script/Pal.PalGameLocalSettings]\nDedicatedServerName={}\n", world);
        result.push(match fs::write(palworld_usersettings_file, settings_file_content) {
            Ok(_) => { ServerSetupCheck::OK { message: format!("Successfully created {}", palworld_usersettings_file.to_str().unwrap()) } },
            Err(_) => { ServerSetupCheck::ERROR { message: format!("Could not write {}", palworld_usersettings_file.to_str().unwrap()) } },
        });
    }
    else {
        result.push(ServerSetupCheck::OK { message: "No saved world, new one will be created on first startup!".to_string() });
    }
    result
}

pub fn init_script(scriptfile: &Path) -> Vec<ServerSetupCheck> {
    if !scriptfile.exists() || !scriptfile.is_file() {
        return vec!(ServerSetupCheck::OK { message: "No init script was found.".to_string() });
    }
    println!("\n\nRunning custom init script...\n\n");
    match Command::new("bash").arg(scriptfile).spawn() {
        Ok(mut c) => {
            match c.wait() {
                Ok(_) => { vec!(ServerSetupCheck::OK { message: "Custom init script was successful".to_string() }) }
                Err(_) => { vec!(ServerSetupCheck::ERROR { message: "Custom init script failed".to_string() }) }
            }
        },
        Err(_) => { vec!(ServerSetupCheck::ERROR { message: "Custom init script could not be ran".to_string() }) }
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
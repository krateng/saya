mod server;
mod proxy;
mod config;

use std::{env, net::{SocketAddrV6, SocketAddrV4, Ipv6Addr, Ipv4Addr}, process};
use std::fs::{OpenOptions, remove_file};
use std::path::{Path, PathBuf};
use colored::Colorize;

#[derive(Clone)]
pub enum ServerSetupCheck {
    OK {message: String},
    WARNING {message: String},
    ERROR {message: String},
}

fn main() {

    let internal_port = env::var("INTERNAL_SERVER_PORT")
        .expect("Missing environment variable INTERNAL_SERVER_PORT")
        .parse::<u16>()
        .expect("Invalid port number");

    let pal_server_dir: PathBuf = Path::new(&env::var("PALSERVERDIR").expect("Missing environment variable PALSERVERDIR")).to_path_buf();
    let pal_settings_dir: PathBuf = pal_server_dir.join("Pal/Saved/Config/LinuxServer");
    let pal_user_settings_file: PathBuf = pal_settings_dir.join("GameUserSettings.ini");
    let pal_settings_file: PathBuf = pal_settings_dir.join("PalWorldSettings.ini");
    let pal_worlds_folder: PathBuf = pal_server_dir.join("Pal/Saved/SaveGames/0");
    let pal_executable: PathBuf = pal_server_dir.join("Pal/Binaries/Linux/PalServer-Linux-Shipping");

    let saya_config_dir: PathBuf = Path::new("/config").to_path_buf();
    let saya_config_file: PathBuf = saya_config_dir.join("palworld_conf.toml");
    let saya_init_script: PathBuf = saya_config_dir.join("init.sh");

    let issues = vec![
        check_requirements(&pal_worlds_folder),
        config::generate_settings(&saya_config_file, &pal_settings_file),
        config::set_world(&pal_worlds_folder, &pal_user_settings_file),
        config::init_script(&saya_init_script),
    ].concat();
    println!();
    present_issues(&issues, true);

    let listen_addr = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 8211, 0, 0);
    let forward_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, internal_port);

    proxy::run_proxy(listen_addr, forward_addr, &pal_executable);

}

fn check_requirements(worlds_folder: &PathBuf) -> Vec<ServerSetupCheck> {

    let mut checks:Vec<ServerSetupCheck> = Vec::new();

    let datatestfile = worlds_folder.join(".writetest");
    let result = OpenOptions::new().write(true).create(true).open(&datatestfile);
    if result.is_err() {
        checks.push(ServerSetupCheck::ERROR {
            message: format!("Can't write to {}. Make sure it is mounted as writable!", worlds_folder.to_str().unwrap()),
        });
    }
    else {
        let _ = remove_file(datatestfile);
        checks.push(ServerSetupCheck::OK {
            message: format!("Save folder {} is writable", worlds_folder.to_str().unwrap()),
        })
    }

    checks
}


/// If errors are present, present all checks and panic. If last is set to true, present all checks anyway.
/// This is meant to be called at multiple times to fail early if issues would prevent further checks; and the final call is to be made with last=true.
fn present_issues(checks: &Vec<ServerSetupCheck>, last: bool) {

    let mut fail = false;

    if checks.len() == 0 {
        return;
    }
    if checks.iter().any(|x| { match x { ServerSetupCheck::ERROR { .. } => true, _ => false } }) {
        fail = true;
    }

    if fail || last {
        if fail {
            println!("{}", "Saya could not initialize:".bold());
        }
        else {
            println!("{}", "Saya successfully initialized:".bold());
        }

        for check in checks {
            match check {
                ServerSetupCheck::ERROR { message } => {
                    println!("\t❎ {}", message.red());
                }
                ServerSetupCheck::WARNING { message } => {
                    println!("\t⚠️ {}", message.yellow());
                }
                ServerSetupCheck::OK { message } => {
                    println!("\t✅ {}", message.green());
                }
            }
        }
    }
    println!();

    if fail {
        process::exit(1);
    }
}
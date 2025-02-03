mod server;
mod proxy;
mod config;

use std::{
    env, net::{SocketAddrV6, SocketAddrV4, Ipv6Addr, Ipv4Addr}
};
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};


fn main() {

    let internal_port = env::var("INTERNAL_SERVER_PORT")
        .expect("Missing environment variable INTERNAL_SERVER_PORT")
        .parse::<u16>()
        .expect("Invalid port number");

    let pal_server_dir: PathBuf = Path::new(&env::var("PALSERVERDIR").expect("Missing environment variable PALSERVERDIR")).to_path_buf();
    let pal_settings_dir: PathBuf = pal_server_dir.join("./Pal/Saved/Config/LinuxServer");
    let pal_user_settings_file: PathBuf = pal_settings_dir.join("./GameUserSettings.ini");
    let pal_world_settings_file: PathBuf = pal_settings_dir.join("./PalWorldSettings.ini");
    let pal_worlds_folder: PathBuf = pal_server_dir.join("./Pal/Saved/SaveGames/0");
    let pal_executable: PathBuf = pal_server_dir.join("./Pal/Binaries/Linux/PalServer-Linux-Shipping");

    let saya_config_dir: PathBuf = Path::new("/config").to_path_buf();
    let saya_config_file: PathBuf = saya_config_dir.join("palworld_conf.toml");
    let saya_init_script: PathBuf = saya_config_dir.join("init.sh");


    check_requirements(&pal_worlds_folder, &saya_config_file);

    config::generate_settings(&saya_config_file, &pal_world_settings_file);
    config::set_world(&pal_worlds_folder, &pal_user_settings_file);

    config::init_script(&saya_init_script);

    let listen_addr = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 8211, 0, 0);
    let forward_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, internal_port);

    proxy::run_proxy(listen_addr, forward_addr, &pal_executable);

}

fn check_requirements(worlds_folder: &PathBuf, config_file: &PathBuf) {

    let mut all_ok = true;

    let result = OpenOptions::new().read(true).open(config_file);
    if result.is_err() {
        println!("Failed to open config file {:?}. Make sure it exists and is mounted as readable!", config_file);
        all_ok = false;
    }

    let datatestfile = worlds_folder.join(".writetest");
    let result = OpenOptions::new().write(true).create(true).open(datatestfile);
    if result.is_err() {
        println!("Can't write to {:?}. Make sure it is mounted as writable!", worlds_folder);
        all_ok = false;
    }

    if !all_ok {
        panic!("File system requirements were not met.")
    }

}

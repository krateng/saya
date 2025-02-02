mod server;
mod proxy;

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

    check_requirements();

    let listen_addr = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 8211, 0, 0);
    let forward_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, internal_port);

    proxy::run_proxy(listen_addr, forward_addr);

}

fn check_requirements() {
    const SAYA_CONFIG_FILE: &str = "/config/palworld_conf.toml";
    const DATA_FOLDER: &str = "./Pal/Saved/SaveGames";

    let mut all_ok = true;

    let conf_file = Path::new(SAYA_CONFIG_FILE);
    let result = OpenOptions::new().read(true).open(conf_file);
    if result.is_err() {
        println!("Failed to open config file {}. Make sure it exists and is mounted as readable!", SAYA_CONFIG_FILE);
        all_ok = false;
    }

    let datatestfile = Path::new(DATA_FOLDER).join(".writetest");
    let result = OpenOptions::new().write(true).create(true).open(datatestfile);
    if result.is_err() {
        println!("Can't write to {:?}. Make sure it is mounted as writable!", DATA_FOLDER);
        all_ok = false;
    }

    if !all_ok {
        panic!("File system requirements were not met.")
    }

}

use std::env;
use std::process::{Child, Command};

pub fn start_server(port: u16) -> Child {
    let community_server_var: u8 = env::var("COMMUNITY_SERVER")
        .expect("Missing environment variable COMMUNITY_SERVER")
        .parse::<u8>()
        .expect("COMMUNITY_SERVER must be a number");
    let community_server: bool = match community_server_var {
        0 => false,
        1 => true,
        _ => panic!("COMMUNITY_SERVER must be 0 or 1")
    };
    let mut cmd = Command::new("./Pal/Binaries/Linux/PalServer-Linux-Shipping");
    cmd
        .arg(format!("-port={}", port))
        .arg("-useperfthreads")
        .arg("-NoAsyncLoadingThread")
        .arg("-UseMultithreadForDS");
    if community_server {
        cmd.arg("-publiclobby");
    }
    cmd.spawn().expect("Failed to start server process")
}
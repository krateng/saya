use std::collections::HashMap;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket};
use std::path::PathBuf;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use crate::server;


const FULL_REQUEST_LOG: bool = false;
const BUFFER_SIZE: usize = 65535;
const TIMEOUT: Duration = Duration::from_secs(200);
const CHECK_INTERVAL: Duration = Duration::from_secs(30);

fn log(line: &str) {
    println!("[{}] {}", "SAYA PROXY", line);
}
fn proxylog(origin: SocketAddr, in_interface: &UdpSocket, out_interface: &UdpSocket, target: SocketAddr, size: usize) {
    if FULL_REQUEST_LOG {
        println!("[SAYA PROXY OP] {:?} -> [{:?} [[ {} byte ]] {:?}] -> {:?}", origin, in_interface.local_addr().unwrap(), size, out_interface.local_addr().unwrap(), target);
    }
}


pub fn run_proxy(listen_addr: SocketAddrV6, forward_addr: SocketAddrV4, server_exec: &PathBuf) {
    let receive_socket = UdpSocket::bind(SocketAddr::V6(listen_addr))
        .expect("Failed to bind UDP socket");
    receive_socket.set_nonblocking(true)
        .expect("Failed to set non-blocking mode");


    let server_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let last_received: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    let client_map: Arc<Mutex<HashMap<SocketAddr, UdpSocket>>> = Arc::new(Mutex::new(HashMap::new()));
    // maps client socket address to the socket we use to forward requests

    let server_process_clone = Arc::clone(&server_process);
    let last_received_clone = Arc::clone(&last_received);
    let client_map_clone = Arc::clone(&client_map);

    // thread to monitor inactivity and kill the server
    thread::spawn(move || {
        loop {
            thread::sleep(CHECK_INTERVAL);

            let mut last_time = last_received_clone.lock().unwrap();
            if let Some(last) = *last_time {
                if last.elapsed() > TIMEOUT {
                    log(format!("No packets received for {} seconds. Stopping server...", TIMEOUT.as_secs()).as_str());

                    let mut process_lock = server_process_clone.lock().unwrap();
                    if let Some(child) = process_lock.as_mut() {
                        let pid = Pid::from_raw(child.id() as i32);
                        // let _ = child.kill().expect("Failed to kill child process");
                        let _ = kill(pid, Signal::SIGINT);
                        let _ = child.wait();
                        log("Server stopped!");
                        *process_lock = None;
                    }
                    *last_time = None;
                }
            }
        }
    });

    // forwarder
    let mut buf = [0u8; BUFFER_SIZE];
    loop {
        match receive_socket.recv_from(&mut buf) {
            Ok((size, src_addr)) => {
                *last_received.lock().unwrap() = Some(Instant::now());


                // start server if down
                let mut process_lock = server_process.lock().unwrap();
                if process_lock.is_none() {
                    log("Received request, starting server...");
                    *process_lock = Some(server::start_server(server_exec, forward_addr.port()));
                    thread::sleep(Duration::from_secs(6));
                    log("Continuing now!");
                }

                // get a socket from which to forward to the server, as well as listen to responses
                // extra block so the guard goes out of scope
                let mut client_map_lock = client_map_clone.lock().unwrap();
                // if socket for this client exists, just send; otherwise create new and start the listening thread
                let send_socket = client_map_lock.entry(src_addr).or_insert_with(|| {
                    let sock = UdpSocket::bind("0.0.0.0:0").expect("Failed to create send socket");
                    sock.set_nonblocking(true).expect("Failed to set non-blocking mode");
                    let response_receive_socket = sock.try_clone().expect("Failed to clone socket");
                    let response_forward_socket = receive_socket.try_clone().expect("Failed to clone socket");
                    let response_target_address = src_addr.clone();
                    thread::spawn(move || {
                        log(format!("Started new response listener on {:?} for {:?}", response_receive_socket, response_target_address).as_str());
                        let mut buf = [0u8; BUFFER_SIZE];
                        loop {
                            match response_receive_socket.recv_from(&mut buf) {
                                Ok((size, response_src_addr)) => {
                                    proxylog(response_src_addr, &response_receive_socket, &response_forward_socket, response_target_address, size);
                                    let _ = response_forward_socket.send_to(&buf[..size], response_target_address);
                                },
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                    thread::sleep(Duration::from_millis(100));
                                },
                                Err(_e) => {
                                }
                            }
                        }
                        // TODO: kill old response listeners after inactivity
                    });
                    sock
                });

                // actually forward our request now
                proxylog(src_addr, &receive_socket, &send_socket, SocketAddr::from(forward_addr), size);
                let _ = send_socket.send_to(&buf[..size], SocketAddr::V4(forward_addr)).expect("Failed to forward packet");

            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            },
            Err(_e) => {
            }
        }
    }
}
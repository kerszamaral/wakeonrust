use crate::delays::CHECK_DELAY;
use crate::packets::{check_packet, make_header, BUFFER_SIZE, SSR_ACK_PACKET, SSR_PACKET};
use crate::pcinfo::{PCInfo, PCStatus};
use crate::ports::MONITOR_PORT;
use crate::signals::Signals;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::{atomic::Ordering, mpsc::Sender, Mutex};

mod listen {
    use super::*;

    fn response_from_client(signals: &Signals, socket: &UdpSocket) -> PCStatus {
        while signals.run.load(Ordering::Relaxed) {
            let mut buf = [0; BUFFER_SIZE];
            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => {
                    if !check_packet(&buf[..amt], SSR_ACK_PACKET) {
                        continue; // Ignore invalid packets
                    }
                    return PCStatus::Online;
                }
                Err(_) => {
                    return PCStatus::Offline;
                }
            }
        }
        PCStatus::Offline
    }

    pub fn listen_for_clients(
        signals: &Signals,
        socket: &UdpSocket,
        pcs: Vec<(&String, &IpAddr, &PCStatus)>,
        sleep_status: &Sender<(String, PCStatus)>,
    ) {
        let ssr = make_header(SSR_PACKET, 0);
        for (hostname, ip, status) in pcs {
            if !signals.run.load(Ordering::Relaxed) {
                break;
            }
            let addr = SocketAddr::new(*ip, MONITOR_PORT);
            socket
                .send_to(&ssr, addr)
                .expect("Failed to send to client");

            let new_status = response_from_client(&signals, &socket);
            if new_status == *status {
                continue;
            }
            sleep_status.send((hostname.clone(), new_status)).unwrap();
        }
    }
}

pub fn initialize(
    signals: &Signals,
    pc_map: &Mutex<HashMap<String, PCInfo>>,
    sleep_status: &Sender<(String, PCStatus)>,
) {
    let addr = SocketAddr::from(([127, 0, 0, 1], MONITOR_PORT));
    let socket = UdpSocket::bind(addr).expect("Failed to bind monitor socket");
    socket
        .set_nonblocking(true)
        .expect("Failed to set monitor socket to non-blocking");
    socket
        .set_read_timeout(Some(CHECK_DELAY))
        .expect("Failed to set monitor socket read timeout");

    while signals.run.load(Ordering::Relaxed) {
        if signals.is_manager.load(Ordering::Relaxed) {
            std::thread::sleep(CHECK_DELAY);

            let pc_map = pc_map.lock().unwrap();
            let pcs = pc_map
                .iter()
                .map(|(k, v)| (k, v.get_ip(), v.get_status()))
                .collect();

            listen::listen_for_clients(&signals, &socket, pcs, &sleep_status);
        } else {
            let mut buf = [0; BUFFER_SIZE];
            match socket.recv_from(&mut buf) {
                Ok((amt, src)) => {
                    if !check_packet(&buf[..amt], SSR_PACKET) {
                        continue;
                    }
                    let ssra = make_header(SSR_ACK_PACKET, 0);
                    socket.send_to(&ssra, src).unwrap();
                }
                Err(_) => {}
            }
        }
    }
}
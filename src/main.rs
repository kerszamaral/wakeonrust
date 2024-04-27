mod addrs;
mod delays;
mod packets;
mod pcinfo;
mod signals;
mod subservices;
use crate::subservices::discovery;
use crate::subservices::interface;
use crate::subservices::management;
use crate::subservices::monitoring;
use pcinfo::PCInfo;
use std::collections::HashMap;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;

fn main() {
    let apc_map = Arc::new(Mutex::new(HashMap::new()));
    let (wake_tx, wake_rx) = channel::<String>();
    let (new_pc_tx, new_pc_rx) = channel::<PCInfo>();
    let (sleep_status_tx, sleep_status_rx) = channel::<(String, pcinfo::PCStatus)>();

    let mut thrds = Vec::<std::thread::JoinHandle<()>>::new();

    let signals = Arc::new(signals::Signals::new());

    let sigwrite = Arc::clone(&signals);
    let apc_out = Arc::clone(&apc_map);
    thrds.push(thread::spawn(move || {
        interface::output::write_output(&sigwrite, &apc_out);
    }));

    let sigread = Arc::clone(&signals);
    thrds.push(thread::spawn(move || {
        interface::input::read_input(&sigread, &wake_tx);
    }));

    let sigdisc = Arc::clone(&signals);
    thrds.push(thread::spawn(move || {
        discovery::initialize(&sigdisc, &new_pc_tx);
    }));

    let sigmon = Arc::clone(&signals);
    let apc_mon = Arc::clone(&apc_map);
    thrds.push(thread::spawn(move || {
        monitoring::initialize(&sigmon, &apc_mon, &sleep_status_tx);
    }));

    let sigexit_send = Arc::clone(&signals);
    thrds.push(thread::spawn(move || {
        management::exit::sender(&sigexit_send);
    }));

    let sigexit_recv = Arc::clone(&signals);
    let apc_exit = Arc::clone(&apc_map);
    thrds.push(thread::spawn(move || {
        management::exit::receiver(&sigexit_recv, &apc_exit);
    }));

    let sigwake_send = Arc::clone(&signals);
    let apc_wake = Arc::clone(&apc_map);
    thrds.push(thread::spawn(move || {
        management::wakeup::sender(&sigwake_send, &wake_rx, &apc_wake);
    }));

    let sigupdt_add = Arc::clone(&signals);
    let apc_updt_add = Arc::clone(&apc_map);
    thrds.push(thread::spawn(move || {
        management::update::add_pcs(&sigupdt_add, &apc_updt_add, &new_pc_rx);
    }));

    let sigupdt_stat = Arc::clone(&signals);
    let apc_updt_stat = Arc::clone(&apc_map);
    thrds.push(thread::spawn(move || {
        management::update::update_statuses(&sigupdt_stat, &apc_updt_stat, &sleep_status_rx);
    }));

    for thrd in thrds.into_iter() {
        thrd.join().unwrap();
    }
}

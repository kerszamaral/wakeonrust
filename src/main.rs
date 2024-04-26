mod delays;
mod packets;
mod pcinfo;
mod ports;
mod signals;
mod subservices;
use crate::subservices::discovery;
use crate::subservices::interface;
use crate::subservices::monitoring;
use pcinfo::PCInfo;
use std::collections::HashMap;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;

fn main() {
    let apc_map = Arc::new(Mutex::new(HashMap::new()));
    let (wake_tx, _wake_rx) = channel::<String>();
    let (new_pc_tx, _new_pc_rx) = channel::<PCInfo>();
    let (sleep_status_tx, _sleep_status_rx) = channel::<(String, pcinfo::PCStatus)>();

    let mut thrds = Vec::<std::thread::JoinHandle<()>>::new();

    let signals = Arc::new(signals::Signals::new());

    let apc_write = Arc::clone(&apc_map);
    let sigwrite = Arc::clone(&signals);
    thrds.push(thread::spawn(move || {
        interface::output::write_output(&sigwrite, &apc_write);
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
    let apc_read = Arc::clone(&apc_map);
    thrds.push(thread::spawn(move || {
        monitoring::initialize(&sigmon, &apc_read, &sleep_status_tx);
    }));

    for thrd in thrds.into_iter() {
        thrd.join().unwrap();
    }
}

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
use std::{env, thread};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    
    let start_as_manager = if args.len() > 1 && args[1] == "manager" {
        true
    } else {
        false
    };

    let signals = Arc::new(signals::Signals::new(start_as_manager));
    
    let am_pc_map = Arc::new(Mutex::new(HashMap::new()));
    let (wakeup_tx, wakeup_rx) = channel::<String>();
    let (new_pc_tx, new_pc_rx) = channel::<PCInfo>();
    let (sleep_status_tx, sleep_status_rx) = channel::<(String, pcinfo::PCStatus)>();

    let mut thrds = Vec::<std::thread::JoinHandle<()>>::new();

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    thrds.push(thread::spawn(move || {
        interface::output::write_output(&sigs, &ampc);
    }));

    let sigs = signals.clone();
    thrds.push(thread::spawn(move || {
        interface::input::read_input(&sigs, wakeup_tx);
    }));

    let sigs = signals.clone();
    thrds.push(thread::spawn(move || {
        discovery::initialize(&sigs, new_pc_tx);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    thrds.push(thread::spawn(move || {
        monitoring::initialize(&sigs, &ampc, sleep_status_tx);
    }));

    let sigs = signals.clone();
    thrds.push(thread::spawn(move || {
        management::exit::sender(&sigs);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    thrds.push(thread::spawn(move || {
        management::exit::receiver(&sigs, &ampc);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    thrds.push(thread::spawn(move || {
        management::wakeup::sender(&sigs, &ampc, wakeup_rx);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    thrds.push(thread::spawn(move || {
        management::update::add_pcs(&sigs, &ampc, new_pc_rx);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    thrds.push(thread::spawn(move || {
        management::update::update_statuses(&sigs, &ampc, sleep_status_rx);
    }));

    for thrd in thrds.into_iter() {
        thrd.join().unwrap();
    }
}

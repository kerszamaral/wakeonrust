// #![allow(dead_code, unused_imports, unused_variables)]
mod addrs;
mod delays;
mod packets;
mod pcinfo;
mod signals;
mod subservices;
use pcinfo::PCInfo;
use std::collections::HashMap;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::{env, thread};
use subservices::{
    discovery, interface, management, monitoring, replication, replication::UpdateType,
    election,
};

fn main() {
    let args = env::args().collect::<Vec<String>>();

    let start_as_manager = if args.len() > 1 && args[1] == "manager" {
        true
    } else {
        false
    };

    let signals = Arc::new(signals::Signals::new(start_as_manager));

    let sigs = signals.clone();
    let old_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        old_panic(panic_info);
        sigs.exit();
    }));

    let sig = signals.clone();
    ctrlc::set_handler(move || {
        sig.exit();
    })
    .unwrap();

    let am_pc_map = Arc::new(Mutex::new(HashMap::new()));
    let (wakeup_tx, wakeup_rx) = channel::<String>();
    let (new_pc_tx, new_pc_rx) = channel::<PCInfo>();
    let (remove_pc_tx, remove_pc_rx) = channel::<String>();
    let (sleep_status_tx, sleep_status_rx) = channel::<(String, pcinfo::PCStatus)>();
    let (update_tx, update_rx) = channel::<(UpdateType, PCInfo)>();

    let mut thrds = Vec::<std::thread::JoinHandle<()>>::new();

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    thrds.push(thread::spawn(move || {
        interface::output::start(&sigs, &ampc);
    }));

    let sigs = signals.clone();
    thrds.push(thread::spawn(move || {
        interface::input::start(&sigs, wakeup_tx);
    }));

    let sigs = signals.clone();
    thrds.push(thread::spawn(move || {
        election::initialize(&sigs);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    thrds.push(thread::spawn(move || {
        replication::initialize(&sigs, &ampc, update_rx);
    }));

    let sigs = signals.clone();
    thrds.push(thread::spawn(move || {
        discovery::discover(&sigs, new_pc_tx);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    thrds.push(thread::spawn(move || {
        monitoring::status::status_monitor(&sigs, &ampc, sleep_status_tx);
    }));

    let sigs = signals.clone();
    thrds.push(thread::spawn(move || {
        monitoring::exit::exit_monitor(&sigs, remove_pc_tx);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    thrds.push(thread::spawn(move || {
        management::wakeup(&sigs, &ampc, wakeup_rx);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    let rb_update_tx = update_tx.clone();
    thrds.push(thread::spawn(move || {
        management::add_pcs(&sigs, &ampc, new_pc_rx, rb_update_tx);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    let rb_update_tx = update_tx.clone();
    thrds.push(thread::spawn(move || {
        management::update_statuses(&sigs, &ampc, sleep_status_rx, rb_update_tx);
    }));

    let sigs = signals.clone();
    let ampc = am_pc_map.clone();
    let rb_update_tx = update_tx.clone();
    thrds.push(thread::spawn(move || {
        management::remove_pcs(&sigs, &ampc, remove_pc_rx, rb_update_tx);
    }));

    for thrd in thrds.into_iter() {
        thrd.join().unwrap();
    }
}

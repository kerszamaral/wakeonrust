mod ports;
mod delays;
mod pcinfo;
mod signals;
mod subservices;
use pcinfo::PCInfo;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc::channel};
use std::thread;
use subservices::interface;

fn main() {
    //! FOR TESTING PURPOSES ONLY
    let mut pc_map = HashMap::new();
    let pc1 = PCInfo::new(
        "pc1".to_string(),
        "00:00:00:00:00:01".parse().unwrap(),
        "127.0.0.1".parse().unwrap(),
        pcinfo::PCStatus::Online,
        true,
    );

    let pc2 = PCInfo::new(
        "pc2".to_string(),
        "00:00:00:00:00:02".parse().unwrap(),
        "127.0.0.1".parse().unwrap(),
        pcinfo::PCStatus::Offline,
        false,
    );

    pc_map.insert(pc1.get_name().clone(), pc1);
    pc_map.insert(pc2.get_name().clone(), pc2);
    // FOR TESTING PURPOSES ONLY


    let apc_map = Arc::new(Mutex::new(pc_map));
    let (wake_tx, _wake_rx) = channel::<String>();
    let (new_pc_tx, _new_pc_rx) = channel::<PCInfo>();

    
    let mut thrds = Vec::<std::thread::JoinHandle<()>>::new();
    
    let signals = Arc::new(signals::Signals::new());
    
    let apc_write = Arc::clone(&apc_map);
    let sigwrite = Arc::clone(&signals);
    thrds.push(thread::spawn(move || {
        interface::output::write_output(&apc_write, &sigwrite);
    }));
    
    let sigread = Arc::clone(&signals);
    thrds.push(thread::spawn(move || {
        interface::input::read_input(&wake_tx,&sigread);
    }));

    let sigdisc = Arc::clone(&signals);
    thrds.push(thread::spawn(move || {
        subservices::discovery::initialize(&sigdisc, &new_pc_tx);
    }));


    for thrd in thrds.into_iter() {
        thrd.join().unwrap();
    }
}

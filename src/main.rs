mod ports;
mod delays;
mod pcinfo;
mod signals;
mod subservices;
use pcinfo::PCInfo;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

fn main() {
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

    let apc_map = Arc::new(pc_map);

    let mut thrds = Vec::<std::thread::JoinHandle<()>>::new();

    let signals = Arc::new(signals::Signals::new());
    let sig2 = Arc::clone(&signals);
    let sig3 = Arc::clone(&signals);

    thrds.push(thread::spawn(move || {
        subservices::interface::output::write_output(&apc_map, &sig2);
    }));

    thrds.push(thread::spawn(move || {
        subservices::interface::input::read_input(&sig3);
    }));

    for thrd in thrds.into_iter() {
        thrd.join().unwrap();
    }
}

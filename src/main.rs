mod subservices;
mod pcinfo;
mod signals;
mod delays;
use std::collections::HashMap;
use std::thread;
use std::sync::Arc;
use pcinfo::PCInfo;

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

    let signals = Arc::new(signals::Signals::new());
    let sig2 = Arc::clone(&signals);
    let apc_map = Arc::new(pc_map);
    thread::spawn(move || {
        subservices::interface::output::write_output(&apc_map, &sig2);
    });

    thread::sleep(std::time::Duration::from_secs(5));
    signals.update.store(true, std::sync::atomic::Ordering::Relaxed);
    thread::sleep(std::time::Duration::from_secs(5));
    signals.update.store(true, std::sync::atomic::Ordering::Relaxed);
    signals.run.store(false, std::sync::atomic::Ordering::Relaxed);

}

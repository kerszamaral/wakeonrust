pub mod input {
    use crate::signals::Signals;
    use std::sync::mpsc::Sender;
    pub fn read_input(wakeups: &Sender<String>, signals: &Signals) {
        while signals.run.load(std::sync::atomic::Ordering::Relaxed) {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            let args = input.split_whitespace().collect::<Vec<&str>>();

            match args.as_slice() {
                ["exit"] => {
                    signals
                        .run
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                    signals
                        .update
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }
                ["wakeup", hostname] => {
                    if signals.is_manager.load(std::sync::atomic::Ordering::Relaxed) {
                        println!("Sending wakeup to {}", hostname);
                        wakeups.send(hostname.to_string()).unwrap();
                    } else {
                        println!("You are not the manager.");
                    }
                }
                _ => {
                    println!("Invalid input.");
                }
            }
        }
    }
}

pub mod output {
    use std::collections::HashMap;
    use std::sync::{
        Mutex,
        atomic::Ordering
    };
    use crate::{delays::WAIT_DELAY, pcinfo::PCInfo, signals::Signals};

    fn make_entry(name: &str, mac: &str, ip: &str, status: &str) -> String {
        format!("{:<20} {:<21} {:<17} {:<8}\n", name, mac, ip, status)
    }

    fn make_header(is_manager: bool) -> String {
        make_entry(
            &if is_manager { "Hostname *" } else { "Hostname" },
            "MAC Address",
            "IPv4 Address",
            "Status",
        )
    }

    fn entry_to_string(pc_info: &PCInfo) -> String {
        let hostname = if *pc_info.get_is_manager() {
            format!("{} *", pc_info.get_name())
        } else {
            pc_info.get_name().to_string()
        };
        make_entry(
            &hostname,
            &pc_info.get_mac().to_string(),
            &pc_info.get_ip().to_string(),
            &format!("{:?}", pc_info.get_status()),
        )
    }

    fn make_table(m_pc_map: &Mutex<HashMap<String, PCInfo>>, is_manager: bool) -> String {
        let pc_map = m_pc_map.lock().unwrap();
        let mut table = make_header(is_manager);
        for pc_info in pc_map.values() {
            table.push_str(&entry_to_string(pc_info));
        }
        table
    }

    pub fn write_output(pc_map: &Mutex<HashMap<String, PCInfo>>, signals: &Signals) {
        while signals.run.load(Ordering::Relaxed) {
            let is_manager = signals.is_manager.load(Ordering::Relaxed);
            println!("{}", make_table(pc_map, is_manager));
            signals.update.store(false, Ordering::Relaxed);
            while !signals.update.load(Ordering::Relaxed) {
                std::thread::sleep(WAIT_DELAY);
            }
        }
    }
}

pub mod input {
    use crate::{delays::INPUT_DELAY, signals::Signals};
    use std::sync::mpsc::{channel, Receiver, Sender};

    fn async_stdin() -> Receiver<String> {
        let (tx, rx) = channel();
        std::thread::spawn(move || loop {
            let mut input = String::new();
            let bytes = std::io::stdin().read_line(&mut input).unwrap();
            // If the user presses Ctrl-D, the program will exit
            if bytes == 0 {
                input = "exit".to_string();  
            }
            match tx.send(input.trim().to_lowercase()) {
                Ok(_) => {}
                Err(_) => break, // The receiver has been dropped
            }
        });
        rx
    }

    pub fn read_input(signals: &Signals, wakeups: Sender<String>) {
        let stdin = async_stdin();
        while signals.running() {
            std::thread::sleep(INPUT_DELAY);
            let input = match stdin.try_recv() {
                Ok(input) => input,
                Err(_) => continue,
            };
            let args = input.split_whitespace().collect::<Vec<&str>>();

            match args.as_slice() {
                ["exit"] => {
                    signals.exit();
                }
                ["wakeup", hostname] => {
                    if signals.is_manager() {
                        wakeups.send(hostname.to_string()).unwrap();
                    } else {
                        println!("Only the manager can send wakeups");
                    }
                }
                _ => {
                    println!("Command not found");
                }
            }
        }
    }
}

pub mod output {
    use crate::{delays::WAIT_DELAY, pcinfo::PCInfo, signals::Signals};
    use std::collections::HashMap;
    use std::sync::Mutex;

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

    pub fn write_output(signals: &Signals, m_pc_map: &Mutex<HashMap<String, PCInfo>>) {
        while signals.running() {
            let is_manager = signals.is_manager();
            #[cfg(not(debug_assertions))]
            clearscreen::clear().unwrap();
            println!("{}", make_table(m_pc_map, is_manager));

            while signals.running() && !signals.has_update() {
                std::thread::sleep(WAIT_DELAY);
            }
        }
    }
}

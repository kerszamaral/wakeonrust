use crate::signals::Signals;

pub fn read_input(signals: &Signals) {
    while signals.run.load(std::sync::atomic::Ordering::Relaxed) {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        match input.to_lowercase().as_str() {
            "exit" => {
                signals.run.store(false, std::sync::atomic::Ordering::Relaxed);
                signals.update.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            _ => {
                println!("Invalid input. Please enter 'exit' to quit.");
            }
        }
    }
}

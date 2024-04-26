use crate::pcinfo::{PCInfo, PCStatus};
use crate::signals::Signals;
use std::sync::mpsc::Receiver;

pub fn initialize(
    _signals: &Signals,
    _new_pc_rx: &Receiver<PCInfo>,
    _wake_rx: &Receiver<String>,
    _sleep_status_rx: &Receiver<(String, PCStatus)>,
) {
}

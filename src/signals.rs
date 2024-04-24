use std::sync::atomic::AtomicBool;


#[derive(Debug)]
pub struct Signals {
    pub run: AtomicBool,
    pub update: AtomicBool,
    pub is_manager: AtomicBool,
}

impl Signals {
    pub fn new() -> Self {
        Self {
            run: AtomicBool::new(true),
            update: AtomicBool::new(true),
            is_manager: AtomicBool::new(false),
        }
    }
}
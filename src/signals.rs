use std::sync::atomic::AtomicBool;


#[derive(Debug)]
pub struct Signals {
    pub run: AtomicBool,
    pub update: AtomicBool,
    pub is_manager: AtomicBool,
    pub manager_found: AtomicBool,
}

impl Signals {
    pub fn new(start_as_manager: bool) -> Self {
        Self {
            run: AtomicBool::new(true),
            update: AtomicBool::new(false),
            is_manager: AtomicBool::new(start_as_manager),
            manager_found: AtomicBool::new(false),
        }
    }
}
use std::sync::atomic::AtomicBool;

#[derive(Debug)]
pub struct Signals {
    run: AtomicBool,
    update: AtomicBool,
    is_manager: AtomicBool,
    manager_found: AtomicBool,
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

    pub fn exit(&self) {
        self.run.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn running(&self) -> bool {
        self.run.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_manager(&self) -> bool {
        self.is_manager.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn has_update(&self) -> bool {
        self.update
            .compare_exchange(
                true,
                false,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_ok()
    }

    pub fn send_update(&self) {
        self.update
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn manager_found(&self) -> bool {
        self.manager_found
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn found_manager(&self) {
        self.manager_found
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn manager_timed_out(&self) {
        self.manager_found
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

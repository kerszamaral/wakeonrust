use std::sync::atomic::{AtomicBool, AtomicU32};

#[derive(Debug)]
pub struct Signals {
    run: AtomicBool,
    update: AtomicBool,
    is_manager: AtomicBool,
    manager_found: AtomicBool,
    electing: AtomicBool,
    table_version: AtomicU32,
}

impl Signals {
    pub fn new(start_as_manager: bool) -> Self {
        Self {
            run: AtomicBool::new(true),
            update: AtomicBool::new(false),
            is_manager: AtomicBool::new(start_as_manager),
            manager_found: AtomicBool::new(false),
            electing: AtomicBool::new(true),
            table_version: AtomicU32::new(0),
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

    pub fn i_am_manager(&self) {
        self.is_manager
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn relinquish_management(&self) {
        self.is_manager
            .store(false, std::sync::atomic::Ordering::Relaxed);
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

    pub fn lost_manager(&self) {
        self.manager_found
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn electing(&self) -> bool {
        self.electing.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn start_election(&self) {
        self.electing
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn end_election(&self) {
        self.electing
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn current_table_version(&self) -> u32 {
        self.table_version
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn update_table_version(&self) -> u32 {
        self.table_version
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    pub fn overwrite_table_version(&self, version: u32) {
        self.table_version
            .store(version, std::sync::atomic::Ordering::Relaxed);
    }
}

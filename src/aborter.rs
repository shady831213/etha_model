use std::sync::atomic::{AtomicBool, Ordering};
pub struct Aborter(AtomicBool);
impl Aborter {
    pub fn new() -> Self {
        Aborter(AtomicBool::new(false))
    }
    pub fn aborted(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
    pub fn abort(&self) {
        self.0.store(true, Ordering::SeqCst)
    }
}

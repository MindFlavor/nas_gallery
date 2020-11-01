use std::sync::atomic::AtomicUsize;

#[derive(Default, Debug)]
pub struct Statistics {
    pub accesses_root: AtomicUsize,
}

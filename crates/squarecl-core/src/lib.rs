use std::sync::atomic::{AtomicU32, Ordering};

pub mod ir;

/// Dummy code, would use real code in `cubecl`
pub fn new_local_var() -> String {
    static LOCAL_COUNT: AtomicU32 = AtomicU32::new(0);

    format!("local_{}", LOCAL_COUNT.fetch_add(1, Ordering::AcqRel))
}

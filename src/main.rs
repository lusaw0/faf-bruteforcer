use crate::consts::{MAX_NS, MAX_S};
use atomic_time::AtomicInstant;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::LazyLock;
use std::time::Instant;

pub mod helpers;
pub mod consts;
pub mod log;
pub mod test;

static PERFECTS: AtomicI32 = AtomicI32::new(0);
static START_PERFECTS: AtomicI32 = AtomicI32::new(0);
static ATTEMPTS: AtomicI32 = AtomicI32::new(1);
static START_ATTEMPTS: AtomicI32 = AtomicI32::new(0);
static _START_SECS: AtomicI32 = AtomicI32::new(0);
static CUR_SECS: AtomicI32 = AtomicI32::new(0);
static CUR_NSECS: AtomicI32 = AtomicI32::new(0);
static START_NSECS: AtomicI32 = AtomicI32::new(9_999_000);
static TRUE_PERFECTS: AtomicI32 = AtomicI32::new(0);
static START_TIME: LazyLock<AtomicInstant> = LazyLock::new(| | AtomicInstant::new(Instant::now()));

fn main() -> Result<(), Box<dyn std::error::Error>> {
    START_TIME.store(Instant::now(), Ordering::SeqCst);
    log::init_read();

    for s in CUR_SECS.load(Ordering::SeqCst)..=MAX_S {
        CUR_SECS.store(s, Ordering::SeqCst);
        for nsec in (CUR_NSECS.load(Ordering::SeqCst)..=MAX_NS).step_by(1000) {
            helpers::run_libtas(helpers::Mode::Bruteforcing, s, nsec)?;
        }
        CUR_NSECS.store(START_NSECS.load(Ordering::SeqCst), Ordering::SeqCst);
    }
    Ok(())
}
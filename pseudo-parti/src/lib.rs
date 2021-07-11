use serde::{Deserialize, Serialize};
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;
use signal_hook::low_level::signal_name;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub fn say_hello() -> &'static str {
    "hello"
}

pub fn setup_signal_handler(interrupted: Arc<AtomicBool>) -> anyhow::Result<()> {
    let mut signals = Signals::new(&[SIGINT, SIGQUIT, SIGTERM])?;
    thread::spawn(move || {
        for signal in signals.forever() {
            println!(
                "Received signal: {:?}:{}. setting interrupted",
                signal_name(signal),
                signal
            );
            interrupted.store(true, Ordering::Relaxed);
        }
    });
    Ok(())
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Action {}

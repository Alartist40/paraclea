//! System safety monitor — watches RAM usage and can trigger emergency shutdown.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use sysinfo::{MemoryRefreshKind, System};
use tracing::{error, info, warn};

/// Background task that monitors system RAM.
pub struct SafetyMonitor {
    threshold_percent: f32,
}

impl SafetyMonitor {
    pub fn new(threshold_percent: f32) -> Self {
        Self {
            threshold_percent: threshold_percent.clamp(0.0, 100.0),
        }
    }

    pub async fn run(&self, shutdown: &AtomicBool, emergency: &AtomicBool) {
        let mut sys = System::new();
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        info!("Safety monitor active: RAM threshold {:.0}%", self.threshold_percent);

        loop {
            interval.tick().await;
            if shutdown.load(Ordering::Relaxed) {
                info!("Safety monitor shutting down");
                return;
            }

            sys.refresh_memory_specifics(MemoryRefreshKind::everything());
            let total = sys.total_memory() as f64;
            let used = sys.used_memory() as f64;
            let pct = (used / total * 100.0) as f32;

            if pct > self.threshold_percent {
                error!(
                    "EMERGENCY: RAM usage {:.1}% exceeds threshold {:.0}% — triggering emergency stop",
                    pct, self.threshold_percent
                );
                emergency.store(true, Ordering::SeqCst);
                return;
            }

            if pct > self.threshold_percent * 0.8 {
                warn!("RAM usage high: {:.1}%", pct);
            }
        }
    }
}

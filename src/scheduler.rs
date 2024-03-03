// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::meter::{Measurements, Meter};

pub struct Task {
    meter: Box<dyn Meter>,
    period: Duration,
    last: Instant,
}

impl Task {
    pub fn new(meter: Box<dyn Meter>, period: Duration) -> Self {
        Self {
            meter,
            period,
            last: Instant::now() - Duration::from_secs(86400), // a long time ago
        }
    }
}

pub struct Scheduler {
    ch: mpsc::SyncSender<Measurements>,
    refresh_period: Duration,
    tasks: Vec<Task>,
}

impl Scheduler {
    pub fn new(ch: mpsc::SyncSender<Measurements>, period: Duration) -> Self {
        Self {
            ch,
            refresh_period: period,
            tasks: Vec::new(),
        }
    }

    pub fn register_task(&mut self, task: Task) {
        log::info!("register {}", task.meter.id());
        self.tasks.push(task);
    }

    pub fn start(&mut self, mut meter_map: Measurements) {
        log::info!("start scheduler");
        let mut last_refresh = Instant::now() - Duration::from_secs(86400); // a long time ago;

        loop {
            let now = Instant::now();

            // Collect stats from meters
            for task in &mut self.tasks {
                let m = &mut task.meter;

                // if timer expired, run our task
                if task.last.elapsed() >= task.period {
                    task.last = now;
                    let val = match m.measure() {
                        Ok(val) => val,
                        Err(err) => {
                            log::warn!("measurement error: {}", err);
                            0.0
                        }
                    };

                    if let Some(slot) = meter_map.get_mut(&m.id()) {
                        *slot = val;
                    }
                }
            }

            // Send state to renderer
            if last_refresh.elapsed() >= self.refresh_period {
                match self.ch.try_send(meter_map.clone()) {
                    Ok(_) => {}
                    Err(err) => {
                        log::info!("scheduler send error: {err}");
                    }
                }
                last_refresh = now;
            }

            thread::sleep(Duration::from_millis(100));
        }
    }
}

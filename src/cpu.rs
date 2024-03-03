// SPDX-License-Identifier: GPL-3.0-or-later

use psutil::{cpu, sensors};

use crate::meter::Meter;
use crate::Res;

// CPU percentage

#[derive(Debug)]
pub struct CpuPercentage {
    pub id: u64,
    cpc: cpu::CpuPercentCollector,
}

impl CpuPercentage {
    pub fn new(id: u64) -> Res<Self> {
        Ok(Self {
            id,
            cpc: cpu::CpuPercentCollector::new()?,
        })
    }
}

impl Meter for CpuPercentage {
    fn id(&self) -> u64 {
        self.id
    }

    fn measure(&mut self) -> Res<f32> {
        let val: f32 = self.cpc.cpu_percent()?;
        Ok(val)
    }
}

// CPU temperature

#[derive(Debug)]
pub struct CpuTemperature {
    pub id: u64,
}

impl CpuTemperature {
    pub fn new(id: u64) -> Res<Self> {
        Ok(Self { id })
    }
}

impl Meter for CpuTemperature {
    fn id(&self) -> u64 {
        self.id
    }

    fn measure(&mut self) -> Res<f32> {
        let temps = sensors::temperatures();
        for temp in temps {
            match &temp {
                Ok(t) => {
                    if t.unit() == "k10temp" && t.label() == Some("Tccd1") {
                        return Ok(t.current().celsius() as f32);
                    }
                }
                Err(_) => (),
            };
        }

        Ok(0.0)
    }
}

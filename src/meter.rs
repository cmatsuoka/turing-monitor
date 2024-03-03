// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use crate::themes::DeviceMeter;
use crate::Res;

pub trait Meter {
    fn id(&self) -> u64;
    fn measure(&mut self) -> Res<f32>;
}

#[derive(Debug, Clone)]
pub struct MeterConfig {
    pub id: u64,
    pub interval: u32,
    pub layout: DeviceMeter,
}

pub type Measurements = HashMap<u64, f32>;

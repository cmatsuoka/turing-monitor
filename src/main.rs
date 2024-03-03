// SPDX-License-Identifier: GPL-3.0-or-later

use std::error::Error;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use clap::Parser;
use simple_logger::SimpleLogger;
use xxhash_rust::const_xxh3::xxh3_64 as const_xxh3;

use crate::cpu::*;
use crate::meter::{Measurements, Meter, MeterConfig};
use crate::render::Renderer;
use crate::scheduler::{Scheduler, Task};

mod cpu;
mod meter;
mod render;
mod scheduler;
mod themes;

type Res<T> = Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(name = "turing-screen")]
#[command(about = "A lightweight turing smart screen updater")]
struct Args {
    /// Set screen brightness in 0-255 range
    #[arg(short, long, value_name = "level")]
    brightness: Option<i32>,

    /// Screen refresh period in seconds
    #[arg(short, long, value_name = "num", default_value_t = 5)]
    refresh: u64,

    /// Serial device to use
    #[arg(short, long, value_name = "device", default_value_t = String::from("AUTO"))]
    port: String,

    #[arg(value_name = "theme_name")]
    theme: String,
}

fn main() {
    let args = Args::parse();

    match run(args) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init()?;

    let refresh_period = Duration::from_secs(args.refresh);
    let theme_name = args.theme;
    let theme = themes::load(&theme_name)?;

    log::info!("using theme: {theme_name}");

    let _scr = turing_screen::new("AUTO")?;

    let mut measurements = Measurements::new();
    let configs = themes::get_meter_list(&theme);
    for cfg in &configs {
        measurements.insert(cfg.id, 0.0);
    }

    // Image rendering thread: prepare framebuffer and communicate
    // with device.
    let (tx, rx) = mpsc::sync_channel(1);
    let renderer_configs = configs.clone();
    thread::spawn(move || {
        let mut renderer = Renderer::new(rx, renderer_configs);
        renderer.start();
    });

    // Main loop: collect pc stats.
    let mut scheduler = Scheduler::new(tx, refresh_period);
    register_meters(&mut scheduler, configs);
    scheduler.start(measurements);

    Ok(())
}

fn register_meters(scheduler: &mut Scheduler, configs: Vec<MeterConfig>) {
    for cfg in configs {
        match create_meter(cfg.id) {
            Ok(m) => {
                let interval = Duration::from_secs(cfg.interval.into());
                scheduler.register_task(Task::new(m, interval));
            }
            Err(err) => {
                log::warn!("cannot register {}: {}", cfg.id, err);
            }
        }
    }
}

const CPU_PERCENTAGE: u64 = const_xxh3(b"CPU:PERCENTAGE");
const CPU_TEMPERATURE: u64 = const_xxh3(b"CPU:TEMPERATURE");

fn create_meter(id: u64) -> Result<Box<dyn Meter>, Box<dyn Error>> {
    let m: Box<dyn Meter> = match id {
        CPU_PERCENTAGE => Box::new(CpuPercentage::new(id)?),
        CPU_TEMPERATURE => Box::new(CpuTemperature::new(id)?),
        _ => return Err("invalid meter".into()),
    };

    Ok(m)
}

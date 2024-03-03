// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::io::BufReader;

use bevy_reflect::{Reflect, Struct};
use serde::Deserialize;
use xxhash_rust::xxh3::xxh3_64;

use crate::meter::MeterConfig;
use crate::Res;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct Display {
    display_orientation: String,
    display_rgb_led: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct Background {
    path: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct StaticImages {
    background: Background,
}

#[derive(Debug, Clone, Default, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct Text {
    pub show: bool,
    pub show_unit: bool,
    pub x: u32,
    pub y: u32,
    pub font: String,
    pub font_size: u32,
    pub font_color: String,
    pub background_color: Option<String>,
    pub background_image: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct Graph {
    show: bool,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    min_value: String,
    max_value: u32,
    bar_color: String,
    bar_outline: bool,
    background_color: Option<String>,
    background_image: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct DeviceMeter {
    interval: Option<u32>,
    pub text: Option<Text>,
    pub graph: Option<Graph>,
}

#[derive(Debug, Clone, Default, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct DeviceStats {
    pub interval: Option<u32>,
    pub percentage: Option<DeviceMeter>,
    pub frequency: Option<DeviceMeter>,
    pub temperature: Option<DeviceMeter>,
}

#[derive(Debug, Clone, Default, Deserialize, Reflect)]
#[serde(rename_all = "UPPERCASE")]
pub struct Stats {
    pub interval: Option<u32>,
    pub cpu: Option<DeviceStats>,
    pub gpu: Option<DeviceStats>,
    //pub memory: Option<MemoryStat>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    display: Display,
    static_images: StaticImages,
    #[serde(alias = "STATS")]
    pub stats: Stats,
}

pub fn load(name: &str) -> Res<Theme> {
    let filepath = format!("res/themes/{}/theme.yaml", name);
    let theme: Theme = load_yaml(&filepath)?;

    // TODO: check theme compatibility

    Ok(theme)
}

fn load_yaml<T>(filename: &str) -> Res<T>
where
    T: serde::de::DeserializeOwned,
{
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let result: T = serde_yaml::from_reader(reader)?;
    Ok(result)
}

pub fn get_meter_list(theme: &Theme) -> Vec<MeterConfig> {
    // The default interval.
    let mut interval = 2;

    // If stats specify a new interval, use it.
    if let Some(val) = theme.stats.interval {
        interval = val;
    }

    let mut res = Vec::<MeterConfig>::new();

    for (i, dev) in theme.stats.iter_fields().enumerate() {
        let mut dev_interval = interval;
        let dev_name = theme.stats.name_at(i).unwrap().to_uppercase();
        if let Some(Some(dev_field)) = dev.downcast_ref::<Option<DeviceStats>>() {
            // If device specifies a local interval, use it.
            if let Some(val) = dev_field.interval {
                dev_interval = val;
            }

            // Iterate over device types.
            for (j, meter) in dev_field.iter_fields().enumerate() {
                let mut meter_interval = dev_interval;
                let meter_name = dev_field.name_at(j).unwrap().to_uppercase();
                if let Some(Some(meter_field)) = meter.downcast_ref::<Option<DeviceMeter>>() {
                    // If meter specifies a local interval, use it.
                    if let Some(val) = meter_field.interval {
                        meter_interval = val;
                    }

                    // Add to list of existing meters.
                    res.push(MeterConfig {
                        id: xxh3_64(format!("{dev_name}:{meter_name}").as_bytes()),
                        interval: meter_interval,
                        layout: meter_field.clone(),
                    });
                }
            }
        }
    }

    res
}

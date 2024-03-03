// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::mpsc;

use crate::meter::{Measurements, MeterConfig};
use crate::themes;
use crate::Res;

pub struct Renderer<'a> {
    ch: mpsc::Receiver<Measurements>,
    widgets: HashMap<u64, themes::DeviceMeter>,
    font: HashMap<String, rusttype::Font<'a>>,
}

impl Renderer<'_> {
    pub fn new(ch: mpsc::Receiver<Measurements>, configs: Vec<MeterConfig>) -> Self {
        let mut widgets = HashMap::<u64, themes::DeviceMeter>::new();
        let mut font_map = HashMap::<String, rusttype::Font>::new();
        for cfg in configs {
            widgets.insert(cfg.id, cfg.layout.clone());
            if let Some(text) = cfg.layout.text {
                let font_path = format!("res/fonts/{}", text.font);
                if font_map.contains_key(font_path.as_str()) {
                    continue;
                }

                log::info!("load font {}", font_path);
                let data = std::fs::read(&font_path).unwrap();
                let font = rusttype::Font::try_from_vec(data).unwrap();
                font_map.insert(font_path.to_owned(), font);
            }
        }
        Self {
            ch,
            widgets,
            font: font_map,
        }
    }

    pub fn start(&mut self) {
        loop {
            match self.ch.recv() {
                Ok(measurements) => {
                    self.render(measurements);
                }
                Err(err) => {
                    log::warn!("renderer receive error: {err}");
                }
            }
        }
    }

    fn render(&self, measurements: Measurements) {
        log::debug!("measurements: {:?}", measurements);
        for (id, value) in measurements {
            render_widget(&self.widgets[&id], value);
        }
    }
}

fn render_widget(widget: &themes::DeviceMeter, value: f32) {
    if let Some(w) = &widget.text {
        render_text(w, 2, value);
    } else if let Some(w) = &widget.graph {
        render_graph(w, value);
    }
}

fn render_text(_text: &themes::Text, field_size: usize, value: f32) {
    let s = format!("{:>size$.*}", 0, value, size = field_size);
    log::debug!("    Text: {}", s);
}

fn render_graph(_graph: &themes::Graph, value: f32) {
    log::debug!("    Graph: {}", value);
}

fn load_font(font_path: &str) -> Res<rusttype::Font> {
    log::info!("load font {}", font_path);
    let font_path = std::env::current_dir()?.join(font_path);
    let data = std::fs::read(font_path)?;
    Ok(rusttype::Font::try_from_vec(data).unwrap())
}

// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::sync::mpsc;

use turing_screen::{Coord, Image, Rect, Rgba, Screen};

use crate::fonts;
use crate::meter::{Measurements, MeterConfig};
use crate::themes;
use crate::Res;

pub struct Renderer<'a> {
    ch: mpsc::Receiver<Measurements>,
    widgets: HashMap<u64, themes::DeviceMeter>,
    font: HashMap<String, fonts::Font<'a>>,
    scr: Box<dyn Screen>,
    bg: Image,
}

impl Renderer<'_> {
    pub fn new(ch: mpsc::Receiver<Measurements>, configs: Vec<MeterConfig>) -> Res<Self> {
        let mut widgets = HashMap::<u64, themes::DeviceMeter>::new();
        let mut font_map = HashMap::<String, fonts::Font>::new();
        for cfg in configs {
            widgets.insert(cfg.id, cfg.layout.clone());
            if let Some(text) = cfg.layout.text {
                let font_path = format!("res/fonts/{}", text.font);

                // don't load fonts twice
                if font_map.contains_key(&text.font) {
                    continue;
                }

                log::info!("load font {}", font_path);
                let data = std::fs::read(&font_path)?;
                let font = fonts::Font::from_data(data)?;
                font_map.insert(text.font, font);
            }
        }

        let mut scr = turing_screen::new("AUTO")?;
        scr.init()?;
        scr.screen_on()?;
        scr.set_brightness(5)?;

        let (width, height) = scr.screen_size();

        log::debug!("framebuffer size: {width}x{height}");
        let bg = Image::new(width, height);

        let renderer = Self {
            ch,
            widgets,
            font: font_map,
            scr,
            bg,
        };

        Ok(renderer)
    }

    pub fn start(&mut self) -> Res<()> {
        let bitmap = lodepng::decode32_file("res/themes/Digital_cpu/background_digital.png")?;
        let mut bg = Image {
            buffer: bitmap.buffer,
            width: bitmap.width,
            height: bitmap.height,
        };

        let rect = Rect::new(0, 0, bg.width, bg.height);
        bg.render_on(&mut self.scr, &rect, &Coord::new(0, 0));
        self.bg.copy_image(&bg, &rect, &Coord::new(0, 0));

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

    fn render(&mut self, measurements: Measurements) {
        log::debug!("measurements: {:?}", measurements);
        for (id, value) in measurements {
            self.render_widget(id, value);
        }
    }

    fn render_widget(&mut self, id: u64, value: f32) -> Res<()> {
        let widget = self.widgets[&id].clone();
        if let Some(w) = &widget.text {
            self.render_text(w, 3, value)?;
        } else if let Some(w) = &widget.graph {
            Self::render_graph(w, value)?;
        }

        Ok(())
    }

    fn render_text(&mut self, text: &themes::Text, field_size: usize, value: f32) -> Res<()> {
        let s = format!("{:>size$.*}", 0, value, size = field_size);
        log::debug!("    Text: {}", s);

        if !&self.font.contains_key(&text.font) {
            return Err(format!("font not loaded: {}", text.font).into());
        }

        let font = &self.font[&text.font];
        let size = text.font_size as f32 * 110.0 / 200.0;
        let color = Rgba::new(0xff, 0, 0, 0xff); // text.font_color;
        let pos = Coord::new(text.x as usize, text.y as usize);

        let (text_img, bb_rect) = fonts::draw_text(&self.bg, font, size, color, &pos, &s);
        let scr = &mut self.scr;
        text_img.render_on(scr, &bb_rect, &pos)?;

        Ok(())
    }

    fn render_graph(_graph: &themes::Graph, value: f32) -> Res<()> {
        log::debug!("    Graph: {}", value);
        Ok(())
    }
}

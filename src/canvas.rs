use std::io::{self, BufWriter, Write};
use std::{fs::File, path::Path};

use crate::color::Color;

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub pixels: Vec<Color>,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        Canvas {
            w,
            h,
            pixels: vec![Color::BLACK; w * h],
        }
    }

    pub fn save_image(&self, filename: impl AsRef<Path>) -> Result<(), io::Error> {
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "P3")?;
        writeln!(writer, "{} {}", self.w, self.h)?;
        writeln!(writer, "255")?;
        for pixel in &self.pixels {
            let r = (pixel.r * 255.0).clamp(0.0, 255.0) as u8;
            let g = (pixel.g * 255.0).clamp(0.0, 255.0) as u8;
            let b = (pixel.b * 255.0).clamp(0.0, 255.0) as u8;
            writeln!(writer, "{r} {g} {b}")?;
        }

        Ok(())
    }
}

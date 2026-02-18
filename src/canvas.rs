use std::io::{self, BufWriter, Write};
use std::{fs::File, path::Path};

use crate::color::{self, Color};

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
            pixels: vec![color::BLACK; w * h],
        }
    }

    // Convert from x and y values (range [0, W-1] and [0, H-1]) to a linear array index
    // fn to_array_coords(&self, x: usize, y: usize) -> usize {
    //     self.w * y + x
    // }

    // These methods are useless now because we're directly setting values of the `pixels` vector
    // pub fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
    //     let coords = self.to_array_coords(x, y);
    //     self.pixels[coords] = color;
    // }

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

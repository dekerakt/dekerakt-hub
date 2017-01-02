use std::ops::{Deref, DerefMut};

pub struct Color {
    r: u8,
    g: u8,
    b: u8
}

impl Color {
    fn rgb(&self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32)
    }

    fn delta(&self, other: &Color) -> f32 {
        let dr = (self.r as f32) - (other.r as f32);
        let dg = (self.g as f32) - (other.g as f32);
        let db = (self.b as f32) - (other.b as f32);
        0.2126 * dr * dr + 0.7152 * dg * dg + 0.0722 * db * db
    }
}

pub struct Palette {
    colors: Vec<Color>
}

impl Palette {
    pub fn new() -> Palette {
        let mut palette = Palette {
            colors: Vec::with_capacity(256)
        };
        palette.generate_shades();
        palette.generate_palette();
        palette
    }

    fn generate_shades(&mut self) {
        for i in 0..16 {
            let shade: u8 = 0xff * (i + 1) / 17;
            let color = Color {
                r: shade,
                g: shade,
                b: shade
            };
            if self.colors.len() >= 16 {
                self[i as usize] = color;
            } else {
                self.colors.push(color);
            }
        }
    }

    fn generate_palette(&mut self) {
        for i in 0..240 {
            let r = i % 6;
            let g = (i / 6) % 8;
            let b = i / (6 * 8);
            let r: u8 = (r * 255 + 2) / 5;
            let g: u8 = (g * 255 + 3) / 7;
            let b: u8 = (b * 255 + 2) / 4;
            let color = Color { r: r, g: g, b: b };
            if self.colors.len() == 256 {
                self[(i + 16) as usize] = color;
            } else {
                self.colors.push(color);
            }
        }
    }
}

impl Deref for Palette {
    type Target = Vec<Color>;

    fn deref(&self) -> &Vec<Color> {
        &self.colors
    }
}

impl DerefMut for Palette {
    fn deref_mut(&mut self) -> &mut Vec<Color> {
        &mut self.colors
    }
}

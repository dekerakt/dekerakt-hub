use std::ops::{Deref, DerefMut};

#[derive(Eq, PartialEq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8
}

impl Color {
    pub fn rgb(&self) -> u32 {
        (self.r as u32) << 16 | (self.g as u32) << 8 | (self.b as u32)
    }

    pub fn delta(&self, other: &Color) -> f32 {
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

    pub fn generate_shades(&mut self) {
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

    pub fn generate_palette(&mut self) {
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

    pub fn color2index(&self, color: Color) -> u8 {
        if let Some(idx) = self.colors.iter().position(|x| *x == color) {
            return idx as u8;
        }
        let idxR: i32 = ((color.r as f32) * 5.0f32 / 255.0f32 + 0.5f32) as i32;
        let idxG: i32 = ((color.g as f32) * 7.0f32 / 255.0f32 + 0.5f32) as i32;
        let idxB: i32 = ((color.b as f32) * 4.0f32 / 255.0f32 + 0.5f32) as i32;
        let idx = 16 + idxR * 8 * 5 + idxG * 5 + idxB;
        let mut minDelta = color.delta(&self[0]);
        let mut minDIdx = 0;
        for i in 1..16 {
            let delta = color.delta(&self[i]);
            if delta < minDelta {
                minDelta = delta;
                minDIdx = i;
            }
        }
        if color.delta(&self[idx as usize]) < minDelta {
            idx as u8
        } else {
            minDIdx as u8
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

#[derive(Copy, Clone)]
pub struct ScreenResolution {
    width: u8,
    height: u8
}

impl ScreenResolution {
    fn area(&self) -> usize {
        (self.width * self.height) as usize
    }
}

pub enum ScreenState {
    On = 0xff,
    Off = 0x00
}

pub enum PreciseMode {
    Precise = 0xff,
    Imprecise = 0x00
}

#[derive(Copy, Clone)]
pub struct Char {
    fg_idx: u8,
    bg_idx: u8,
    char: char
}

impl Char {
    fn empty() -> Char {
        Char { fg_idx: 255, bg_idx: 16, char: ' ' }
    }
}

pub struct Canvas {
    canvas: Vec<Char>,
    resolution: ScreenResolution,
    fg: u8,
    bg: u8,
    palette: Palette
}

impl Canvas {
    pub fn new(palette: Palette, resolution: ScreenResolution) -> Canvas {
        let mut canvas = Canvas {
            canvas: Vec::with_capacity(resolution.area()),
            resolution: resolution,
            fg: 255,
            bg: 16,
            palette: palette
        };
        canvas.initial_chars();
        canvas
    }

    fn initial_chars(&mut self) {
        if self.canvas.len() != 0 {
            return;
        }
        for i in 0..self.resolution.area() {
            self.canvas.push(Char::empty());
        }
    }

    fn resize(&mut self, resolution: ScreenResolution) {
        let mut newCanvas: Vec<Char> = Vec::with_capacity(resolution.area());
        for y in 0..resolution.height {
            for x in 0..resolution.width {
                let c: Char;
                if y < self.resolution.height && x < self.resolution.width {
                    c = *self.get(x, y);
                } else {
                    c = Char::empty();
                }
                newCanvas.push(c);
            }
        }
        self.canvas = newCanvas;
        self.resolution = resolution;
    }

    fn get(&self, x: u8, y: u8) -> &Char {
        if x > self.resolution.width || y > self.resolution.height {
            panic!("invalid values for x and y: larger than resolution");
        }
        &self.canvas[(y * self.resolution.width + x) as usize]
    }

    fn set(&mut self, x: u8, y: u8, char: Char) {
        if x > self.resolution.width || y > self.resolution.height {
            // no-op
            return;
        }
        self.canvas[(y * self.resolution.width + x) as usize] = char;
    }

    fn copy(&mut self, x: u8, y: u8, mut w: u8, mut h: u8, tx: u8, ty: u8) {
        if x + w > self.resolution.width {
            w = self.resolution.width - x;
        }
        if y + h > self.resolution.height {
            h = self.resolution.height - y;
        }
        let mut copy_region: Vec<Char> = Vec::with_capacity((w * h) as usize);
        for j in y..(y + h) {
            for i in x..(x + w) {
                copy_region.push(*self.get(i, j));
            }
        }
        for j in 0..h {
            for i in 0..w {
                self.set(tx + i, ty + j, copy_region[(j * w + i) as usize]);
            }
        }
    }
}

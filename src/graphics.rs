use std::ops::{Deref, DerefMut};
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Default, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

pub type Palette = [Color; 16];

#[derive(Debug, Copy, Clone)]
pub struct ScreenResolution {
    pub width: u8,
    pub height: u8
}

impl ScreenResolution {
    pub fn area(&self) -> usize {
        (self.width * self.height) as usize
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ScreenState {
    On = 0xff,
    Off = 0x00
}

#[derive(Debug, Copy, Clone)]
pub enum PreciseMode {
    Precise = 0xff,
    Imprecise = 0x00
}

#[derive(Debug, Copy, Clone)]
pub struct Char {
    pub fg_idx: u8,
    pub bg_idx: u8,
    pub char: char
}

impl Char {
    fn empty() -> Char {
        Char { fg_idx: 255, bg_idx: 16, char: ' ' }
    }
}

#[derive(Debug, Clone)]
pub struct Canvas {
    pub canvas: Vec<Char>,
    pub resolution: ScreenResolution,
    pub fg: u8,
    pub bg: u8
}

impl Canvas {
    pub fn new(resolution: ScreenResolution) -> Canvas {
        let mut canvas = Canvas {
            canvas: Vec::with_capacity(resolution.area()),
            resolution: resolution,
            fg: 255,
            bg: 16
        };
        canvas.initial_chars();
        canvas
    }

    fn initial_chars(&mut self) {
        if self.canvas.len() != 0 {
            return;
        }
        for _ in 0..self.resolution.area() {
            self.canvas.push(Char::empty());
        }
    }

    pub fn resize(&mut self, resolution: ScreenResolution) {
        let mut new_canvas: Vec<Char> = Vec::with_capacity(resolution.area());
        for y in 0..resolution.height {
            for x in 0..resolution.width {
                let c: Char;
                if y < self.resolution.height && x < self.resolution.width {
                    c = *self.get(x, y);
                } else {
                    c = Char::empty();
                }
                new_canvas.push(c);
            }
        }
        self.canvas = new_canvas;
        self.resolution = resolution;
    }

    pub fn get(&self, x: u8, y: u8) -> &Char {
        if x > self.resolution.width || y > self.resolution.height {
            panic!("invalid values for x and y: larger than resolution");
        }
        &self.canvas[(y * self.resolution.width + x) as usize]
    }

    pub fn set(&mut self, x: u8, y: u8, char: Char) {
        if x > self.resolution.width || y > self.resolution.height {
            // no-op
            return;
        }
        self.canvas[(y * self.resolution.width + x) as usize] = char;
    }

    pub fn copy(&mut self, x: u8, y: u8, mut w: u8, mut h: u8, tx: u8, ty: u8) {
        if x > self.resolution.width || y > self.resolution.height {
            return;
        }
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
                self.set(x + tx + i, y + ty + j, copy_region[(j * w + i) as usize]);
            }
        }
    }

    pub fn fill(&mut self, x: u8, y: u8, mut w: u8, mut h: u8, c: char) {
        if x > self.resolution.width || y > self.resolution.height {
            return;
        }
        if x + w > self.resolution.width {
            w = self.resolution.width - x;
        }
        if y + h > self.resolution.height {
            h = self.resolution.height - y;
        }
        let char = Char {
            char: c,
            fg_idx: self.fg,
            bg_idx: self.bg
        };
        for j in y..(y + h) {
            for i in x..(x + w) {
                self.set(i, j, char);
            }
        }
    }

    pub fn set_string(&mut self, mut x: u8, mut y: u8, chars: String,
                      vertical: bool)
    {
        if x > self.resolution.width || y > self.resolution.height {
            return;
        }
        for c in chars.chars() {
            let width = UnicodeWidthChar::width(c).unwrap() as u8;
            let char = Char {
                char: c,
                fg_idx: self.fg,
                bg_idx: self.bg
            };
            if width > 1 {
                let space = Char {
                    char: ' ',
                    fg_idx: self.fg,
                    bg_idx: self.bg
                };
                for i in 1..width {
                    self.set(x + i, y, space);
                }
            }
            self.set(x, y, char);
            if vertical {
                y += 1;
            } else {
                x += width;
            }
        }
    }
}


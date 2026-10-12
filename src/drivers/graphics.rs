const CHAR_W: usize = 8;
const CHAR_H: usize = 16;

const FONT: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/console/font.bin"));

pub struct Framebuffer {
    pub buffer: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bpp: usize,
}

unsafe impl Send for Framebuffer {}

impl Framebuffer {
    pub const fn new(buffer: *mut u8, width: usize, height: usize, pitch: usize, bpp: usize) -> Self {
        Self { buffer, width, height, pitch, bpp }
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = y * self.pitch + x * (self.bpp / 8);

        unsafe {
            let ptr = self.buffer.add(offset);
            core::ptr::write_volatile(ptr, (color & 0xFF) as u8);
            core::ptr::write_volatile(ptr.add(1), ((color >> 8) & 0xFF) as u8);
            core::ptr::write_volatile(ptr.add(2), ((color >> 16) & 0xFF) as u8);
            core::ptr::write_volatile(ptr.add(3), ((color >> 24) & 0xFF) as u8);
        }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        for row in 0..h {
            for col in 0..w {
                self.put_pixel(x + col, y + row, color);
            }
        }
    }

    pub fn fill_screen(&mut self, color: u32) {
        self.fill_rect(0, 0, self.width, self.height, color)
    }
}

pub struct GraphicsConsole {
    fb: Framebuffer,
    cursor_x: usize,
    cursor_y: usize,
}

impl GraphicsConsole {
    pub fn new(fb: Framebuffer) -> Self {
        Self { fb, cursor_x: 0, cursor_y: 0 }
    }

    pub fn cols(&self) -> usize {
        self.fb.width / CHAR_W
    }

    pub fn rows(&self) -> usize {
        self.fb.height / CHAR_H
    }

    pub fn clear(&mut self, bg: u32) {
        self.fb.fill_screen(bg);
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    pub fn draw_glyph(&mut self, cx: usize, cy: usize, ch: u8, fg: u32, bg: u32) {
        let px = cx * CHAR_W;
        let py = cy * CHAR_H;
        let base = ch as usize * CHAR_H;

        for row in 0..CHAR_H {
            let bits = FONT[base + row];
            for col in 0..CHAR_W {
                let color = if bits & (0x80 >> col) != 0 { fg } else { bg };
                self.fb.put_pixel(px + col, py + row, color);
            }
        }
    }

    pub fn scroll_up(&mut self, bg: u32) {
        let shift = CHAR_H * self.fb.pitch;
        let total = self.fb.height * self.fb.pitch;
        unsafe {
            core::ptr::copy(
                self.fb.buffer.add(shift),
                self.fb.buffer,
                total - shift,
            );
        }
        self.fb.fill_rect(0, self.fb.height - CHAR_H, self.fb.width, CHAR_H, bg);
    }

    pub fn print_char(&mut self, ch: u8, fg: u32, bg: u32) {
        match ch {
            b'\n' => {
                self.cursor_x = 0;
                self.cursor_y += 1;
            }
            b'\r' => {
                self.cursor_x = 0;
            }
            _ => {
                if self.cursor_x >= self.cols() {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                }
                self.draw_glyph(self.cursor_x, self.cursor_y, ch, fg, bg);
                self.cursor_x += 1;
            }
        }

        if self.cursor_y >= self.rows() {
            self.scroll_up(bg);
            self.cursor_y = self.rows() - 1;
        }
    }

    pub fn print_str(&mut self, text: &str, fg: u32, bg: u32) {
        for &b in text.as_bytes() {
            self.print_char(b, fg, bg);
        }
    }

    pub fn backspace(&mut self, bg: u32) {
        if self.cursor_x == 0 && self.cursor_y == 0 {
            return;
        }

        if self.cursor_x == 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.cols() - 1;
        } else {
            self.cursor_x -= 1;
        }

        self.draw_glyph(self.cursor_x, self.cursor_y, b' ', bg, bg);
    }
}
pub struct Framebuffer {
    buffer: *mut u8,
    width: usize,
    height: usize,
    pitch: usize,
    bpp: usize,
}

impl Framebuffer {
    pub fn new(buffer: *mut u8, width: usize, height: usize, pitch: usize, bpp: usize) -> Self {
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
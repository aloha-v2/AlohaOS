pub struct SerialPort {
    base: u16,
}

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

impl SerialPort {
    pub const COM1: u16 = 0x3F8;

    pub const fn new(base: u16) -> Self {
        Self { base }
    }

    pub fn init(&self) {
        unsafe {
            outb(self.base + 1, 0x00);
            outb(self.base + 3, 0x80);
            outb(self.base + 0, 0x01);
            outb(self.base + 1, 0x00);
            outb(self.base + 3, 0x03);
            outb(self.base + 2, 0xC7);
            outb(self.base + 4, 0x0B);
        }
    }

    pub fn write_byte(&self, byte: u8) {
        unsafe {
            while inb(self.base + 5) & 0x20 == 0 {}
            outb(self.base + 0, byte);
        }
    }

    pub fn write_str(&self, text: &str) {
        for &b in text.as_bytes() {
            if b == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(b);
        }
    }
}
#![no_std]
#![no_main]

struct VgaWriter {
    x: usize,
    y: usize,
    color: u8,
    buffer: *mut u8,
}

impl VgaWriter {
    fn new(color: u8) -> VgaWriter {
        VgaWriter {
            x: 0,
            y: 0,
            color,
            buffer: 0xb8000 as *mut u8,
        }
    }

    fn print_char(&mut self, c: u8) {
        match c {
            b'\n' => {
                self.x = 0;
                self.y += 1;
                return;
            }
            b'\r' => {
                self.x = 0;
                return;
            }
            _ => {}
        }

        let index = (self.y * WIDTH + self.x) * 2;

        unsafe {
            core::ptr::write_volatile(self.buffer.add(index), c);
            core::ptr::write_volatile(self.buffer.add(index + 1), self.color);
        }

        self.x += 1;

        if self.x == WIDTH {
            self.x = 0;
            self.y += 1;
        }
    }

    fn print_str(&mut self, text: &str) {
        for byte in text.bytes() {
            self.print_char(byte);
        }
    }
}

const WIDTH: usize = 80;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut writer = VgaWriter::new(0x0F);
    writer.print_str("Hello from AlohaOS!\n");

    writer.color = 0x0A;
    writer.print_str("Kernel is alive!");
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
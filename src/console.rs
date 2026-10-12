use crate::arch::x86_64::serial::SerialPort;
use crate::drivers::graphics::{Framebuffer, GraphicsConsole};
use crate::sync::spinlock::SpinLock;

pub const FG: u32 = 0x00FFFFFF;
pub const BG: u32 = 0x00000000;

static SERIAL: SpinLock<SerialPort> = SpinLock::new(SerialPort::new(SerialPort::COM1));
static CONSOLE: SpinLock<Option<GraphicsConsole>> = SpinLock::new(None);

pub fn init() {
    SERIAL.lock().init();
}

pub fn init_console(fb: Framebuffer) {
    let mut console = GraphicsConsole::new(fb);
    console.clear(BG);
    *CONSOLE.lock() = Some(console);
}

pub fn print(text: &str) {
    SERIAL.lock().write_str(text);
    let mut guard = CONSOLE.lock();
    if let Some(console) = guard.as_mut() {
        console.print_str(text, FG, BG);
    }
}

pub fn print_usize(n: usize) {
    if n == 0 {
        print("0");
        return;
    }

    let mut buf = [0u8; 20];
    let mut pos = buf.len();
    let mut n = n;

    while n > 0 {
        pos -= 1;
        buf[pos] = (n % 10) as u8 + b'0';
        n /= 10;
    }

    if let Ok(s) = core::str::from_utf8(&buf[pos..]) {
        print(s);
    }
}

pub fn print_char(ch: u8) {
    let serial = SERIAL.lock();
    if ch == b'\n' {
        serial.write_byte(b'\r');
    }
    serial.write_byte(ch);
    drop(serial);

    let mut guard = CONSOLE.lock();
    if let Some(console) = guard.as_mut() {
        console.print_char(ch, FG, BG);
    }
}

pub fn backspace() {
    SERIAL.lock().write_str("\x08 \x08");

    let mut guard = CONSOLE.lock();
    if let Some(console) = guard.as_mut() {
        console.backspace(BG);
    }
}


pub fn emergency_print(text: &str) {
    SerialPort::new(SerialPort::COM1).write_str(text);
}

pub fn emergency_hex(n: u64) {
    let s = SerialPort::new(SerialPort::COM1);
    s.write_str("0x");
    for i in (0..16).rev() {
        let d = ((n >> (i * 4)) & 0xF) as u8;
        s.write_byte(if d < 10 { b'0' + d } else { b'a' + d - 10 });
    }
}

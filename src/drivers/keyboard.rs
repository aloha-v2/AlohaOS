use crate::arch::x86_64::port::Port;

pub struct Keyboard {
    port: Port,
}

impl Keyboard {
    pub const fn new() -> Self {
        Self {
            port: Port::new(0x60),
        }
    }

    pub fn read_scancode(&self) -> u8 {
        self.port.read()
    }
}

const fn build_map() -> [u8; 256] {
    let mut m = [0u8; 256];
    m[0x1E] = b'a';
    m[0x30] = b'b';
    m[0x2E] = b'c';
    m[0x20] = b'd';
    m[0x12] = b'e';
    m[0x21] = b'f';
    m[0x22] = b'g';
    m[0x23] = b'h';
    m[0x17] = b'i';
    m[0x24] = b'j';
    m[0x25] = b'k';
    m[0x26] = b'l';
    m[0x32] = b'm';
    m[0x31] = b'n';
    m[0x18] = b'o';
    m[0x19] = b'p';
    m[0x10] = b'q';
    m[0x13] = b'r';
    m[0x1F] = b's';
    m[0x14] = b't';
    m[0x16] = b'u';
    m[0x2F] = b'v';
    m[0x11] = b'w';
    m[0x2D] = b'x';
    m[0x15] = b'y';
    m[0x2C] = b'z';
    m[0x0B] = b'0';
    m[0x02] = b'1';
    m[0x03] = b'2';
    m[0x04] = b'3';
    m[0x05] = b'4';
    m[0x06] = b'5';
    m[0x07] = b'6';
    m[0x08] = b'7';
    m[0x09] = b'8';
    m[0x0A] = b'9';
    m[0x39] = b' ';
    m[0x1C] = b'\n';
    m[0x0E] = b'\x08';
    m
}

const SCANCODE_MAP: [u8; 256] = build_map();

pub fn scancode_to_ascii(sc: u8) -> Option<u8> {
    let c = SCANCODE_MAP[sc as usize];
    if c == 0 { None } else { Some(c) }
} 
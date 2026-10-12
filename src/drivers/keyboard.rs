use crate::arch::x86_64::port::Port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(u8),
    Enter,
    Backspace,
    ShiftLeft,
    ShiftRight,
    CapsLock,
    Tab,
    Space,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    Pressed(KeyCode), // клавиша нажата
    Released(KeyCode), // клавиша отпущена
}

pub struct Keyboard {
    port: Port,
    shift_count: u8,
    caps_lock: bool,
    extended: bool,
}

impl Keyboard {
    pub const fn new() -> Self {
        Self {
            port: Port::new(0x60),
            shift_count: 0,
            caps_lock: false,
            extended: false,
        }
    }

    pub fn process_scancode(&mut self, sc: u8) -> Option<KeyEvent> {
        if sc == 0xE0 {
            self.extended = true;
            return None;
        }

        if self.extended == true {
            self.extended = false;
            return None;
        }

        let released = (sc & 0x80) != 0;
        
        let code = sc & 0x7F;

        match code {
            0x2A => {
                if released {
                    self.shift_count = self.shift_count.saturating_sub(1);
                    return Some(KeyEvent::Released(KeyCode::ShiftLeft));
                } else {
                    self.shift_count += 1;
                    return Some(KeyEvent::Pressed(KeyCode::ShiftLeft));
                }
            }
            0x36 => {
                if released {
                    self.shift_count = self.shift_count.saturating_sub(1);
                    return Some(KeyEvent::Released(KeyCode::ShiftRight));
                } else {
                    self.shift_count += 1;
                    return Some(KeyEvent::Pressed(KeyCode::ShiftRight));
                }
            }
            0x3A => {
                if !released {
                    self.caps_lock = !self.caps_lock;
                    return Some(KeyEvent::Pressed(KeyCode::CapsLock));
                } else {
                    return Some(KeyEvent::Released(KeyCode::CapsLock));
                }
            }
            _ => {}
        }

        if released {
            return None;
        }

        match code {
            0x1C => return Some(KeyEvent::Pressed(KeyCode::Enter)),
            0x0E => return Some(KeyEvent::Pressed(KeyCode::Backspace)),
            0x0F => return Some(KeyEvent::Pressed(KeyCode::Tab)),
            0x01 => return Some(KeyEvent::Pressed(KeyCode::Escape)),
            0x39 => return Some(KeyEvent::Pressed(KeyCode::Space)),
            _ => {}
        }

        let shift = self.shift_count > 0;
        let normal = NORMAL_MAP[code as usize];

        let c = if normal.is_ascii_lowercase() {
            // буквы: Shift и Caps Lock инвертируют друг друга
            if shift ^ self.caps_lock { normal.to_ascii_uppercase() } else { normal }
        } else if shift {
            SHIFTED_MAP[code as usize]
        } else {
            normal
        };

        if c == 0 {
            return None;
        }

        return Some(KeyEvent::Pressed(KeyCode::Char(c)));
    }

    #[allow(dead_code)]
    pub fn read_scancode(&self) -> u8 {
        self.port.read()
    }
}

// ---------- Инициализация контроллера PS/2 (i8042) ----------
// После UEFI/OVMF контроллер может остаться с выключенным IRQ1 или с байтом
// в выходном буфере; тогда прерывание клавиатуры не придёт никогда.

const STATUS_CMD: u16 = 0x64;
const DATA: u16 = 0x60;

fn wait_write() -> bool {
    let s = Port::new(STATUS_CMD);
    for _ in 0..100_000 {
        if s.read() & 0x02 == 0 { return true; }
    }
    false
}

fn wait_read() -> bool {
    let s = Port::new(STATUS_CMD);
    for _ in 0..100_000 {
        if s.read() & 0x01 != 0 { return true; }
    }
    false
}

fn flush_output() {
    let s = Port::new(STATUS_CMD);
    let d = Port::new(DATA);
    for _ in 0..32 {
        if s.read() & 0x01 != 0 { d.read(); } else { break; }
    }
}

pub fn init_controller() {
    let cmd = Port::new(STATUS_CMD);
    let data = Port::new(DATA);

    if wait_write() { cmd.write(0xAD); }          // временно выключить порт клавиатуры
    flush_output();

    if wait_write() { cmd.write(0x20); }          // прочитать config byte
    let mut cfg = if wait_read() { data.read() } else { 0x00 };
    cfg |= 0b0100_0001;                           // bit0: IRQ1 вкл, bit6: трансляция в set 1
    cfg &= !0b0001_0000;                          // bit4: тактирование клавиатуры вкл
    if wait_write() { cmd.write(0x60); }          // записать config byte
    if wait_write() { data.write(cfg); }

    if wait_write() { cmd.write(0xAE); }          // включить порт клавиатуры

    if wait_write() { data.write(0xF4); }         // enable scanning
    if wait_read() { data.read(); }               // ответ ACK (0xFA)
    flush_output();
}

const NORMAL_MAP: [u8; 128] = [
    0,    0,    b'1', b'2', b'3', b'4', b'5', b'6', // 0x00
    b'7', b'8', b'9', b'0', b'-', b'=', 0,    0,    // 0x08
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', // 0x10
    b'o', b'p', b'[', b']', 0,    0,    b'a', b's', // 0x18
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', // 0x20
    b'\'',b'`', 0,    b'\\',b'z', b'x', b'c', b'v', // 0x28
    b'b', b'n', b'm', b',', b'.', b'/', 0,    b'*', // 0x30
    0,    b' ', 0,    0,    0,    0,    0,    0,    // 0x38
    0,    0,    0,    0,    0,    0,    0,    b'7', // 0x40
    b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', // 0x48
    b'2', b'3', b'0', b'.', 0,    0,    0,    0,    // 0x50
    0,    0,    0,    0,    0,    0,    0,    0,    // 0x58
    0,    0,    0,    0,    0,    0,    0,    0,    // 0x60
    0,    0,    0,    0,    0,    0,    0,    0,    // 0x68
    0,    0,    0,    0,    0,    0,    0,    0,    // 0x70
    0,    0,    0,    0,    0,    0,    0,    0,    // 0x78
];

const SHIFTED_MAP: [u8; 128] = [
    0,    0,    b'!', b'@', b'#', b'$', b'%', b'^', // 0x00
    b'&', b'*', b'(', b')', b'_', b'+', 0,    0,    // 0x08
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', // 0x10
    b'o', b'p', b'{', b'}', 0,    0,    b'a', b's', // 0x18
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b':', // 0x20
    b'"', b'~', 0,    b'|', b'z', b'x', b'c', b'v', // 0x28
    b'b', b'n', b'm', b'<', b'>', b'?', 0,    b'*', // 0x30
    0,    b' ', 0,    0,    0,    0,    0,    0,    // 0x38
    0,    0,    0,    0,    0,    0,    0,    b'7', // 0x40
    b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', // 0x48
    b'2', b'3', b'0', b'.', 0,    0,    0,    0,    // 0x50
    0,    0,    0,    0,    0,    0,    0,    0,    // 0x58
    0,    0,    0,    0,    0,    0,    0,    0,    // 0x60
    0,    0,    0,    0,    0,    0,    0,    0,    // 0x68
    0,    0,    0,    0,    0,    0,    0,    0,    // 0x70
    0,    0,    0,    0,    0,    0,    0,    0,    // 0x78
];
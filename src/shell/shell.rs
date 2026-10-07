use crate::drivers::keyboard::{KeyCode, KeyEvent};
use crate::SpinLock;

struct InputBuffer {
    data: [u8; 256],
    len: usize,
}

static INPUT_BUFFER: SpinLock<InputBuffer> = SpinLock::new(InputBuffer::new());

impl InputBuffer {
    pub const fn new() -> Self {
        Self {
            data: [0u8; 256],
            len: 0,
        }
    }

    fn push(&mut self, c: u8) {
        if self.len < 256 {
            erase_cursor();
            self.data[self.len] = c;
            self.len += 1;
            if let Ok(s) = core::str::from_utf8(&[c]) {
                crate::print(s);
            }
            draw_cursor();
        }
    }

    fn pop(&mut self) {
        if self.len > 0 {
            erase_cursor();
            self.len -= 1;
            crate::print("\x08");
            draw_cursor();
        }
    }

    fn clear(&mut self) {
        self.len = 0
    }

    fn as_str(&self) -> Option<&str> {
        let slice = &self.data[..self.len];
        core::str::from_utf8(slice).ok()
    }
}

pub fn draw_cursor() {
    crate::print("_");
}

fn erase_cursor() {
    crate::print("\x08");
}

pub fn handle_key_event(event: KeyEvent) {
    match event {
        KeyEvent::Pressed(keycode) => {
            match keycode {
                KeyCode::Char(c) => {
                    let mut buffer = INPUT_BUFFER.lock();
                    buffer.push(c);
                }
                KeyCode::Enter => {
                    let mut buffer = INPUT_BUFFER.lock();
                    erase_cursor();
                    buffer.clear();
                    crate::print("\n");
                    draw_cursor();
                }
                KeyCode::Backspace => {
                    let mut buffer = INPUT_BUFFER.lock();
                    buffer.pop();
                }
                KeyCode::Space => {
                    let mut buffer = INPUT_BUFFER.lock();
                    buffer.push(b' ');
                }
                KeyCode::Tab => {
                    let mut buffer = INPUT_BUFFER.lock();
                    for _ in 0..4 {
                        buffer.push(b' ');
                    }
                }
                KeyCode::Escape => {},
                _ => {}
            }
        }
        KeyEvent::Released(_) => {}
    }
}
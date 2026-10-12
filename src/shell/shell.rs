use crate::drivers::keyboard::{KeyEvent, KeyCode};
use crate::sync::ring_buffer::RingBuffer;

static EVENT_QUEUE: RingBuffer = RingBuffer::new();

pub fn push_event(event: KeyEvent) {
    EVENT_QUEUE.push(event);
}

pub fn poll_event() -> Option<KeyEvent> {
    EVENT_QUEUE.pop()
}

pub fn handle_key_event(event: KeyEvent) {
    if let KeyEvent::Pressed(keycode) = event {
        match keycode {
            KeyCode::Char(c) => crate::console::print_char(c),
            KeyCode::Enter => crate::console::print_char(b'\n'),
            KeyCode::Backspace => crate::console::backspace(),
            KeyCode::Space => crate::console::print_char(b' '),
            KeyCode::Tab => {
                for _ in 0..4 { crate::console::print_char(b' '); }
            }
            _ => {}
        }
    }
}

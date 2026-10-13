use crate::drivers::keyboard::{KeyEvent, KeyCode};
use crate::sync::ring_buffer::RingBuffer;
use crate::sync::spinlock::SpinLock;
use crate::shell::input::InputBuffer;
use crate::shell::commands;
use crate::shell::commands::CommandEffect;

static EVENT_QUEUE: RingBuffer = RingBuffer::new();
static INPUT_BUFFER: SpinLock<InputBuffer> = SpinLock::new(InputBuffer::new());

enum ShellState {
    Normal,
    Confirming(fn() -> !),
}

static SHELL_STATE: SpinLock<ShellState> = SpinLock::new(ShellState::Normal);

pub fn push_event(event: KeyEvent) {
    EVENT_QUEUE.push(event);
}

pub fn poll_event() -> Option<KeyEvent> {
    EVENT_QUEUE.pop()
}

pub fn handle_key_event(event: KeyEvent) {
    if let KeyEvent::Pressed(keycode) = event {
        match keycode {
            KeyCode::Char(c) => {
                if INPUT_BUFFER.lock().push(c) {
                    crate::console::print_char(c);
                }
            }
            KeyCode::Enter => {
                crate::console::print_char(b'\n');
                handle_enter();
            }
            KeyCode::Backspace => {
                if INPUT_BUFFER.lock().pop().is_some() {
                    crate::console::backspace();
                }
            }
            KeyCode::Space => {
                if INPUT_BUFFER.lock().push(b' ') {
                    crate::console::print_char(b' ');
                }
            }
            KeyCode::Tab => {
                for _ in 0..4 {
                    if INPUT_BUFFER.lock().push(b' ') {
                        crate::console::print_char(b' ');
                    }
                }
            }
            _ => {}
        }
    }
}

fn handle_enter() {
    let state = {
        let mut guard = SHELL_STATE.lock();
        core::mem::replace(&mut *guard, ShellState::Normal)
    };

    match state {
        ShellState::Normal => {
            let effect = {
                let mut guard = INPUT_BUFFER.lock();
                let effect = commands::execute(guard.as_str());
                guard.clear();
                effect
            };

            match effect {
                CommandEffect::None => print_prompt(),
                CommandEffect::Confirm(action) => {
                    *SHELL_STATE.lock() = ShellState::Confirming(action);
                }
            }
        }

        ShellState::Confirming(action) => {
            let (is_yes, is_no) = {
                let mut guard = INPUT_BUFFER.lock();
                let is_yes = guard.as_str().eq_ignore_ascii_case("yes");
                let is_no = guard.as_str().eq_ignore_ascii_case("no");
                guard.clear();
                (is_yes, is_no)
            };

            if is_yes {
                action();
            } else if is_no {
                crate::console::print("Cancelled.\n");
                print_prompt();
            } else {
                crate::console::print("Please answer yes or no: ");
                *SHELL_STATE.lock() = ShellState::Confirming(action);
            }
        }
    }
}

pub fn print_prompt() {
    use crate::console::{BG, FG, GREEN};
    crate::console::set_color(GREEN, BG);
    crate::console::print("aloha");
    crate::console::set_color(FG, BG);
    crate::console::print("$ ");
}

pub fn init() {
    print_prompt();
}
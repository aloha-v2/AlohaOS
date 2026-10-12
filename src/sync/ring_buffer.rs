use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use crate::drivers::keyboard::{KeyEvent, KeyCode};

const SIZE: usize = 64;

/// Кольцевой буфер "один производитель (ISR) / один потребитель (main loop)".
pub struct RingBuffer {
    buffer: UnsafeCell<[KeyEvent; SIZE]>,
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl RingBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: UnsafeCell::new([KeyEvent::Released(KeyCode::Escape); SIZE]),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, event: KeyEvent) {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        let next_head = (head + 1) % SIZE;
        if next_head == tail {
            return; // буфер полон — событие теряется
        }

        unsafe {
            (*self.buffer.get())[head] = event;
        }

        self.head.store(next_head, Ordering::Release);
    }

    pub fn pop(&self) -> Option<KeyEvent> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        if tail == head {
            return None;
        }

        let event = unsafe { (*self.buffer.get())[tail] };

        self.tail.store((tail + 1) % SIZE, Ordering::Release);

        Some(event)
    }
}

unsafe impl Sync for RingBuffer {}

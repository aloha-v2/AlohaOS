use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use crate::drivers::keyboard::{KeyEvent, KeyCode};

pub struct RingBuffer {
    buffer: UnsafeCell<[KeyEvent; 64]>,
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl RingBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: UnsafeCell::new([KeyEvent::Released(KeyCode::Escape); 64]),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, event: KeyEvent) {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);


        let next_head = (head + 1) % 64;
        if next_head == tail {
            return;
        }

        let ptr = self.buffer.get();
        unsafe {
            (*ptr)[head] = event;
        }

        self.head.store(next_head, Ordering::Relaxed);
    }

    pub fn pop(&self) -> Option<KeyEvent> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);

        if tail == head {
            return None;
        }

        let ptr = self.buffer.get();
        let event = unsafe {
            (*ptr)[tail]
        };

        let next_tail = (tail + 1) % 64;
        self.tail.store(next_tail, Ordering::Relaxed);
        
        return Some(event);
    }
}

unsafe impl Sync for RingBuffer {}
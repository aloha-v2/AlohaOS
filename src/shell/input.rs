pub struct InputBuffer {
    data: [u8; 256],
    len: usize,
}

impl InputBuffer {
    pub const fn new() -> Self {
        Self { 
            data: [0; 256],
            len: 0,
        }
    }

    pub fn push(&mut self, byte: u8) -> bool {
        if self.len < 256 {
            self.data[self.len] = byte;
            self.len += 1;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.data[self.len])
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
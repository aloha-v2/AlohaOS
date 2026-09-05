pub struct Port {
    port: u16,
}

impl Port {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
        }
    }

    pub fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            core::arch::asm!(
                "in al, dx",
                out("al") value,
                in("dx") self.port,
                options(nostack, nomem, preserves_flags),
            );
        }
        value
    }

    pub fn write(&self, value: u8) {
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("al") value,
                in("dx") self.port,
                options(nostack, nomem, preserves_flags),
            );
        }
    }
}
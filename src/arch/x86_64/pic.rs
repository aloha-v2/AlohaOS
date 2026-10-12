use super::port::Port;

pub struct Pic {
    master_command: Port, // 0x20
    master_data: Port, // 0x21
    slave_command: Port, // 0xA0
    slave_data: Port, // 0xA1
}

pub static PIC: Pic = Pic::new();

/// Небольшая задержка между командами PIC (нужна на реальном железе).
fn io_wait() {
    Port::new(0x80).write(0);
}

impl Pic {
    pub const fn new() -> Self {
        Self {
            master_command: Port::new(0x20),
            master_data: Port::new(0x21),
            slave_command: Port::new(0xA0),
            slave_data: Port::new(0xA1),
        }
    }

    pub fn init(&self) {
        self.master_command.write(0x11); io_wait();
        self.slave_command.write(0x11); io_wait();

        // IRQ 0-7 -> векторы 0x20-0x27, IRQ 8-15 -> 0x28-0x2F
        self.master_data.write(0x20); io_wait();
        self.slave_data.write(0x28); io_wait();

        self.master_data.write(0x04); io_wait();
        self.slave_data.write(0x02); io_wait();

        self.master_data.write(0x01); io_wait();
        self.slave_data.write(0x01); io_wait();

        // Маска: разрешены IRQ0 (таймер) и IRQ1 (клавиатура)
        self.master_data.write(0xFC);
        self.slave_data.write(0xFF);
    }

    pub fn send_eoi(&self, irq: u8) {
        if irq >= 8 {
            self.slave_command.write(0x20);
        }

        self.master_command.write(0x20);
    }
}

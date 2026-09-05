use super::port::Port;

pub struct Pic {
    master_command: Port, // 0x20
    master_data: Port, // 0x21
    slave_command: Port, // 0xA0
    slave_data: Port, // 0xA1
}

pub static PIC: Pic = Pic::new();

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
        self.master_command.write(0x11);
        self.slave_command.write(0x11);

        self.master_data.write(0x20);
        self.slave_data.write(0x28);

        self.master_data.write(0x04);
        self.slave_data.write(0x02);

        self.master_data.write(0x01);
        self.slave_data.write(0x01);

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
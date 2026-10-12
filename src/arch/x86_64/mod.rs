pub mod idt;
pub mod port;
pub mod pic;
pub mod serial;

/// Настраивает IDT, PIC и контроллер PS/2. Прерывания остаются выключенными.
pub fn init() {
    idt::init();
    pic::PIC.init();
    crate::drivers::keyboard::init_controller();
}

pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}

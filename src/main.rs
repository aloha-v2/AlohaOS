#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod arch;
mod boot;
mod console;
mod drivers;
mod shell;
mod sync;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    console::init();
    console::print("AlohaOS booting...\n");

    arch::x86_64::init();
    console::print("IDT, PIC and PS/2 initialized\n");

    if let Some(fb) = boot::framebuffer() {
        console::init_console(fb);
        console::print("Graphics console ready\n");
    }

    arch::x86_64::enable_interrupts();
    console::print("Interrupts enabled. Type something:\n");

    loop {
        arch::x86_64::idt::poll_keyboard();

        while let Some(event) = shell::poll_event() {
            shell::handle_key_event(event);
        }
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    console::emergency_print("\nKernel PANIC! System halted.\n");
    if let Some(loc) = info.location() {
        console::emergency_print(loc.file());
        console::emergency_print(":");
        console::emergency_hex(loc.line() as u64);
        console::emergency_print("\n");
    }
    loop {
        unsafe { core::arch::asm!("cli; hlt"); }
    }
}

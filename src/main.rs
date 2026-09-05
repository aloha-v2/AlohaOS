#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod drivers;
mod arch;

use drivers::vga::VgaWriter;
use arch::x86_64::pic::PIC;

static mut WRITER: Option<VgaWriter> = None;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        WRITER = Some(VgaWriter::new(0x0F));
    }
    print("AlohaOS booting...\n");
    
    arch::x86_64::idt::init();
    print("IDT loaded\n");

    PIC.init();
    unsafe {
        core::arch::asm!("sti");
    }
    print("Interrupts enabled\n");

    loop {}
}

pub fn print(text: &str) {
    unsafe {
        if let Some(ref mut w) = WRITER {
            w.print_str(text);
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    print("\nKernel PANIC! System halted.\n");
    loop {}
}
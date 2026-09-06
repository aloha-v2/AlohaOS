#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod drivers;
mod arch;
mod sync;

use drivers::vga::VgaWriter;
use arch::x86_64::pic::PIC;
use sync::spinlock::SpinLock;

static WRITER: SpinLock<VgaWriter> = SpinLock::new(VgaWriter::new(0x0F));

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
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
    let mut guard = WRITER.lock();
    guard.print_str(text);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    print("\nKernel PANIC! System halted.\n");
    loop {}
}
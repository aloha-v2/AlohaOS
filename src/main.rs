#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod drivers;
mod arch;
mod sync;
mod shell;

use drivers::vga::VgaWriter;
use arch::x86_64::pic::PIC;
use sync::spinlock::SpinLock;
use sync::ring_buffer::RingBuffer;
use shell::shell::draw_cursor;

static EVENT_QUEUE: RingBuffer = RingBuffer::new();
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

    draw_cursor();

    loop {
        if let Some(event) = EVENT_QUEUE.pop()  {
            shell::handle_key_event(event);
        };
        
        unsafe { core::arch::asm!("hlt"); }
    }
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
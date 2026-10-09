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
use limine::request::FramebufferRequest;
use limine::BaseRevision;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

static EVENT_QUEUE: RingBuffer = RingBuffer::new();
static WRITER: SpinLock<VgaWriter> = SpinLock::new(VgaWriter::new(0x0F));

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // print("AlohaOS booting...\n");
    
    arch::x86_64::idt::init();
    // print("IDT loaded\n");

    PIC.init();
    unsafe {
        core::arch::asm!("sti");
    }
    // print("Interrupts enabled\n");

    // draw_cursor();

    if let Some(fb_response) = FRAMEBUFFER_REQUEST.response() {
        if let Some(fb) = fb_response.framebuffers().first() {
            let mut framebuffer = drivers::graphics::Framebuffer::new(
                fb.address() as *mut u8,
                fb.width as usize,
                fb.height as usize,
                fb.pitch as usize,
                fb.bpp as usize,
            );
            framebuffer.fill_screen(0x00FF0000);
        }
    }

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
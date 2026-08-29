#![no_std]
#![no_main]

mod drivers;

use drivers::vga::VgaWriter;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut writer = VgaWriter::new(0x0F);
    writer.print_str("Hello from AlohaOS!\n");

    writer.set_color(0x0A);
    writer.print_str("Kernel is alive!");

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
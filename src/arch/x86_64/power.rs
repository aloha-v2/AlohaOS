pub fn shutdown() -> ! {
    unsafe {
        core::arch::asm!(
            "out dx, ax",
            in("dx") 0x604u16,
            in("ax") 0x2000u16,
            options(nomem, nostack, preserves_flags),
        );
    }
    loop {
        unsafe { core::arch::asm!("cli; hlt"); }
    }
}
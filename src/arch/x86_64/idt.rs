use crate::drivers::keyboard::{Keyboard, KeyCode, KeyEvent};
use crate::sync::spinlock::SpinLock;
use super::pic::PIC;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

pub struct Idt {
    entries: [IdtEntry; 256],
}   

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

static KEYBOARD: SpinLock<Keyboard> = SpinLock::new(Keyboard::new());
static mut IDT: Idt = Idt::new();

impl IdtEntry {
    pub const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    pub fn set_handler(&mut self, handler: u64) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.selector = 0x08;
        self.ist = 0;
        self.type_attr = 0x8E;
        self.offset_mid = ((handler >> 16) & 0xFFFF) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.reserved = 0;
    }
}

impl Idt {
    pub fn set_handler(&mut self, index: u8, handler: u64) {
        self.entries[index as usize].set_handler(handler)
    }

    pub const fn new() -> Self {
        let entries: [IdtEntry; 256] = [IdtEntry::missing(); 256];
        Self {
            entries,
        }
    }

    pub fn load(&self) {
        let idt_ptr = IdtPointer {
            limit: (core::mem::size_of::<Idt>() - 1) as u16,
            base: self as *const Idt as u64,
        };

        unsafe {
            core::arch::asm!(
                "lidt [{}]",
                in(reg) &idt_ptr,
                options(nostack, preserves_flags)
            );
        }
    }
}

pub fn init() {
    use core::ptr::addr_of_mut;

    unsafe {
        let idt_ptr = addr_of_mut!(IDT);

        (*idt_ptr).set_handler(0, divide_error_handler as *const () as u64);
        (*idt_ptr).set_handler(32, timer_handler as *const () as u64);
        (*idt_ptr).set_handler(33, keyboard_handler as *const () as u64);
        (*idt_ptr).load();
    }
}

extern "C" fn divide_error_handler() -> ! {
    crate::print("DIVIDE BY ZERO caught!\n");
    loop {}
}

extern "x86-interrupt" fn timer_handler(_frame: InterruptStackFrame) {
    PIC.send_eoi(0);
}

extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    let mut guard = KEYBOARD.lock();
    let sc = guard.read_scancode();
        
    if let Some(event) = guard.process_scancode(sc) {
        handle_key_event(event);
    }

    PIC.send_eoi(1);
}

fn handle_key_event(event: KeyEvent) {
    match event {
        KeyEvent::Pressed(keycode) => {
            match keycode {
                KeyCode::Char(c) => {
                    if let Ok(s) = core::str::from_utf8(&[c]) {
                        crate::print(s);
                    }
                }
                KeyCode::Enter => crate::print("\n"),
                KeyCode::Backspace => crate::print("\x08"),
                KeyCode::Space => crate::print(" "),
                KeyCode::Tab => crate::print("    "),
                KeyCode::Escape => {},
                _ => {}
            }
        }
        KeyEvent::Released(_) => {}
    }
}
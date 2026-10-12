use crate::drivers::keyboard::Keyboard;
use crate::sync::spinlock::SpinLock;
use super::pic::PIC;
use super::port::Port;

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

/// Текущий селектор кода. Limine ставит свою GDT, где 64-битный код НЕ 0x08,
/// поэтому берём реальный CS, а не хардкодим число.
fn current_cs() -> u16 {
    let cs: u16;
    unsafe {
        core::arch::asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack, preserves_flags));
    }
    cs
}

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

    pub fn set_handler(&mut self, handler: u64, selector: u16) {
        self.offset_low = (handler & 0xFFFF) as u16;
        self.selector = selector;
        self.ist = 0;
        self.type_attr = 0x8E; // present, DPL=0, interrupt gate
        self.offset_mid = ((handler >> 16) & 0xFFFF) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.reserved = 0;
    }
}

impl Idt {
    pub fn set_handler(&mut self, index: u8, handler: u64, selector: u16) {
        self.entries[index as usize].set_handler(handler, selector)
    }

    pub const fn new() -> Self {
        let entries: [IdtEntry; 256] = [IdtEntry::missing(); 256];
        Self { entries }
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

    let cs = current_cs();

    unsafe {
        let idt_ptr = addr_of_mut!(IDT);

        (*idt_ptr).set_handler(0, divide_error_handler as *const () as u64, cs);
        (*idt_ptr).set_handler(8, double_fault_handler as *const () as u64, cs);
        (*idt_ptr).set_handler(13, general_protection_handler as *const () as u64, cs);
        (*idt_ptr).set_handler(14, page_fault_handler as *const () as u64, cs);
        (*idt_ptr).set_handler(32, timer_handler as *const () as u64, cs);
        (*idt_ptr).set_handler(33, keyboard_handler as *const () as u64, cs);
        (*idt_ptr).set_handler(39, spurious_master_handler as *const () as u64, cs);
        (*idt_ptr).set_handler(47, spurious_slave_handler as *const () as u64, cs);
        (*idt_ptr).load();
    }
}

fn halt_forever() -> ! {
    loop {
        unsafe { core::arch::asm!("cli; hlt"); }
    }
}

extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    crate::console::emergency_print("\nEXCEPTION: #DE divide by zero, rip=");
    crate::console::emergency_hex(frame.instruction_pointer);
    crate::console::emergency_print("\n");
    halt_forever();
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _err: u64) -> ! {
    crate::console::emergency_print("\nEXCEPTION: #DF double fault, rip=");
    crate::console::emergency_hex(frame.instruction_pointer);
    crate::console::emergency_print("\n");
    halt_forever();
}

extern "x86-interrupt" fn general_protection_handler(frame: InterruptStackFrame, err: u64) {
    crate::console::emergency_print("\nEXCEPTION: #GP, err=");
    crate::console::emergency_hex(err);
    crate::console::emergency_print(" rip=");
    crate::console::emergency_hex(frame.instruction_pointer);
    crate::console::emergency_print("\n");
    halt_forever();
}

extern "x86-interrupt" fn page_fault_handler(frame: InterruptStackFrame, err: u64) {
    let cr2: u64;
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
    }
    crate::console::emergency_print("\nEXCEPTION: #PF, err=");
    crate::console::emergency_hex(err);
    crate::console::emergency_print(" addr=");
    crate::console::emergency_hex(cr2);
    crate::console::emergency_print(" rip=");
    crate::console::emergency_hex(frame.instruction_pointer);
    crate::console::emergency_print("\n");
    halt_forever();
}

/// Забирает ВСЕ байты, которые сейчас лежат в выходном буфере контроллера PS/2.
/// Вызывать только при выключенных прерываниях (иначе между проверкой статуса
/// и чтением порта 0x60 байт может забрать другой обработчик).
fn drain_controller() {
    let status = Port::new(0x64);
    let data = Port::new(0x60);

    // bit0 = в буфере есть байт, bit5 = байт от мыши (его не трогаем как клавиатурный)
    for _ in 0..16 {
        let st = status.read();
        if st & 0x01 == 0 {
            break;
        }
        let sc = data.read();
        if st & 0x20 != 0 {
            continue; // байт мыши — выбросить
        }
        let event = KEYBOARD.lock().process_scancode(sc);
        if let Some(event) = event {
            crate::shell::push_event(event);
        }
    }
}

/// Запасной путь: опрос контроллера из основного цикла.
/// Работает, даже если IRQ1 по какой-то причине не доходит до процессора.
pub fn poll_keyboard() {
    let flags: u64;
    unsafe {
        core::arch::asm!("pushfq; pop {}", out(reg) flags);
        core::arch::asm!("cli", options(nomem, nostack));
    }

    drain_controller();

    if flags & (1 << 9) != 0 {
        unsafe { core::arch::asm!("sti", options(nomem, nostack)); }
    }
}

extern "x86-interrupt" fn timer_handler(_frame: InterruptStackFrame) {
    // Таймер тоже опрашивает клавиатуру (~18 раз в секунду) как страховку.
    drain_controller();
    PIC.send_eoi(0);
}

extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    drain_controller();
    PIC.send_eoi(1);
}

// Ложные прерывания PIC: на IRQ7 / IRQ15 EOI для ложного не шлём.
extern "x86-interrupt" fn spurious_master_handler(_frame: InterruptStackFrame) {}

extern "x86-interrupt" fn spurious_slave_handler(_frame: InterruptStackFrame) {
    // Мастеру EOI нужен (каскадная линия), слейву — нет.
    PIC.send_eoi(0);
}

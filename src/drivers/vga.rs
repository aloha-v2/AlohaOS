pub struct VgaWriter {
    x: usize,
    y: usize,
    color: u8,
    buffer: *mut u8,
}

// размеры текстового VGA буфера
const WIDTH: usize = 80;
const HEIGHT: usize = 25;

// писатель в текстовый VGA буфер
impl VgaWriter {
    // создает нового писателя с указанным цветом текста
    pub fn new(color: u8) -> VgaWriter {
        VgaWriter {
            x: 0,
            y: 0,
            color,
            buffer: 0xb8000 as *mut u8,
        }
    }

    // Выводит один символ в текущую позицию курсора
    pub fn print_char(&mut self, c: u8) {
        match c {
            b'\n' => {
                self.new_line();
                return;
            }
            b'\r' => {
                self.x = 0;
                return;
            }
            b'\x08' => {
                if self.x > 0 {
                    self.x -= 1;
                    self.write_cell(self.x, self.y, b' ');
                }
                return;
            }
            _ => {}
        }

        self.write_cell(self.x, self.y, c);

        self.x += 1;
        if self.x == WIDTH {
            self.new_line();
        }
    }

    // переходит на новую строку
    fn new_line(&mut self) {
        self.x = 0;
        self.y += 1;
        if self.y == HEIGHT {
            self.scroll_up();
            self.y = HEIGHT - 1;
        }
    }

    // выводит строку текста посимвольно
    pub fn print_str(&mut self, text: &str) {
        for byte in text.bytes() {
            self.print_char(byte);
        }
    }

    // устанавливает текущий цвет текста
    pub fn set_color(&mut self, color: u8) {
        self.color = color;
    }

    // записывает символ и его атрибут цвета
    fn write_cell(&mut self, x: usize, y: usize, c: u8) {
        let index = (y * WIDTH + x) * 2;
        unsafe {
            core::ptr::write_volatile(self.buffer.add(index), c);
            core::ptr::write_volatile(self.buffer.add(index + 1), self.color);
        }
    }


    // прокручивает содержимое экрана на одну строку вверх
    fn scroll_up(&mut self) {
        for row in 0..(HEIGHT - 1) {
            for col in 0..WIDTH {
                let src = ((row + 1) * WIDTH + col) * 2;
                let dst = (row * WIDTH + col) * 2;

                unsafe {
                    let ch = core::ptr::read_volatile(self.buffer.add(src));
                    let co = core::ptr::read_volatile(self.buffer.add(src + 1));
                    core::ptr::write_volatile(self.buffer.add(dst), ch);
                    core::ptr::write_volatile(self.buffer.add(dst + 1), co);
                }
            }
        }

        for col in 0..WIDTH {
            self.write_cell(col, HEIGHT - 1, b' ');
        }
    }
}
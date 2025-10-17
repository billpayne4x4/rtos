use core::fmt::{self, Write};
use crate::framebuffer::Framebuffer;

/// Simple 5×7 console rendered into the linear framebuffer.
pub struct Console<'a> {
    fb: &'a mut Framebuffer,
    origin_x: u32,
    origin_y: u32,
    cursor_x: u32,
    cursor_y: u32,
    cell_w: u32,     // glyph width + spacing
    cell_h: u32,     // glyph height + spacing
    fg: u32,
    bg: Option<u32>,
}

impl<'a> Console<'a> {
    /// Create a console with a 5×7 font (plus 1px spacing).
    pub fn new(fb: &'a mut Framebuffer, fg_rgb: (u8,u8,u8), bg_rgb: Option<(u8,u8,u8)>) -> Self {
        let fg = fb.pack_rgb(fg_rgb.0, fg_rgb.1, fg_rgb.2);
        let bg = bg_rgb.map(|(r,g,b)| fb.pack_rgb(r,g,b));
        Console {
            fb,
            origin_x: 0,
            origin_y: 0,
            cursor_x: 0,
            cursor_y: 0,
            cell_w: 6, // 5px glyph + 1px spacing
            cell_h: 8, // 7px glyph + 1px spacing
            fg,
            bg,
        }
    }

    /// Set top-left drawing origin (useful for side-by-side panes).
    pub fn set_origin(&mut self, x: u32, y: u32) {
        self.origin_x = x;
        self.origin_y = y;
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Set colors.
    pub fn set_colors(&mut self, fg: (u8,u8,u8), bg: Option<(u8,u8,u8)>) {
        self.fg = self.fb.pack_rgb(fg.0, fg.1, fg.2);
        self.bg = bg.map(|(r,g,b)| self.fb.pack_rgb(r,g,b));
    }

    /// Clear the entire console area (the whole framebuffer by default).
    pub fn clear(&mut self) {
        if let Some(bg) = self.bg {
            self.fb.fill_rect(0, 0, self.fb.width, self.fb.height, bg);
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Print a string (no trailing newline).
    pub fn write_str(&mut self, s: &str) {
        let mut w = ConsoleWriter { con: self };
        let _ = w.write_str(s);
    }

    /// Print string + newline.
    pub fn write_line(&mut self, s: &str) {
        let mut w = ConsoleWriter { con: self };
        let _ = writeln!(w, "{}", s);
    }

    /// Draw a single character at the current cursor, then advance.
    pub fn putc(&mut self, ch: char) {
        match ch {
            '\n' => self.newline(),
            '\r' => { /* ignore */ }
            _ => {
                // Wrap horizontally
                if self.pixel_x() + self.cell_w > self.fb.width {
                    self.newline();
                }

                // Optional background for the cell
                if let Some(bg) = self.bg {
                    self.fb.fill_rect(self.pixel_x(), self.pixel_y(), self.cell_w, self.cell_h, bg);
                }

                // Render glyph (5×7)
                let pattern = glyph_5x7(ch);
                for row in 0..7 {
                    let bits = pattern[row as usize];
                    for col in 0..5 {
                        if (bits >> (4 - col)) & 1 == 1 {
                            self.fb.put_pixel(self.pixel_x() + col, self.pixel_y() + row, self.fg);
                        }
                    }
                }

                // Advance cursor
                self.cursor_x += 1;
            }
        }
        // Scroll if needed after printing
        self.ensure_visible();
    }

    fn pixel_x(&self) -> u32 { self.origin_x + self.cursor_x * self.cell_w }
    fn pixel_y(&self) -> u32 { self.origin_y + self.cursor_y * self.cell_h }

    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
    }

    /// If we ran past the bottom line, scroll up by one text row.
    fn ensure_visible(&mut self) {
        let next_y = self.pixel_y() + self.cell_h;
        if next_y <= self.fb.height { return; }
        self.scroll_up(1);
        self.cursor_y = self.cursor_y.saturating_sub(1);
    }

    /// Scroll by N text rows (cell height).
    fn scroll_up(&mut self, rows: u32) {
        if rows == 0 { return; }
        let dy = rows * self.cell_h;
        if dy >= self.fb.height { // big hammer: clear entire screen
            if let Some(bg) = self.bg {
                self.fb.fill_rect(0, 0, self.fb.width, self.fb.height, bg);
            }
            self.cursor_x = 0;
            self.cursor_y = 0;
            return;
        }

        // Raw move: copy framebuffer bytes upward
        let bytes_per_row = (self.fb.stride as usize) * 4;
        unsafe {
            let base = self.fb.ptr as *mut u8;
            let src  = base.add((dy as usize) * bytes_per_row);
            let dst  = base;
            let cnt  = (self.fb.height as usize - dy as usize) * bytes_per_row;
            core::ptr::copy(src, dst, cnt);
        }

        // Clear the vacated area
        if let Some(bg) = self.bg {
            self.fb.fill_rect(0, self.fb.height - dy, self.fb.width, dy, bg);
        }
    }
}

/// Adaptor so Console implements core::fmt::Write ergonomically.
struct ConsoleWriter<'a, 'b> { con: &'b mut Console<'a> }

impl<'a, 'b> Write for ConsoleWriter<'a, 'b> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for ch in s.chars() { self.con.putc(ch); }
        Ok(())
    }
}

/// 5×7 bitmap glyphs (MSB..LSB = left..right).
/// Minimal but practical set; unknown chars map to '?'.
fn glyph_5x7(ch: char) -> [u8; 7] {
    match ch {
        ' ' => [0,0,0,0,0,0,0],
        '!' => [0b00100,0b00100,0b00100,0b00100,0b00100,0,0b00100],
        '.' => [0,0,0,0,0,0b00110,0b00110],
        ',' => [0,0,0,0,0,0b00110,0b00100],
        ':' => [0,0b00110,0b00110,0,0b00110,0b00110,0],
        ';' => [0,0b00110,0b00110,0,0b00110,0b00100,0],
        '-' => [0,0,0,0b11111,0,0,0],
        '_' => [0,0,0,0,0,0,0b11111],
        '/' => [0b00001,0b00010,0b00100,0b01000,0b10000,0,0],
        '\\'=> [0b10000,0b01000,0b00100,0b00010,0b00001,0,0],

        '0' => [0b01110,0b10001,0b10011,0b10101,0b11001,0b10001,0b01110],
        '1' => [0b00100,0b01100,0b00100,0b00100,0b00100,0b00100,0b01110],
        '2' => [0b01110,0b10001,0b00001,0b00010,0b00100,0b01000,0b11111],
        '3' => [0b11110,0b00001,0b00001,0b00110,0b00001,0b00001,0b11110],
        '4' => [0b00010,0b00110,0b01010,0b10010,0b11111,0b00010,0b00010],
        '5' => [0b11111,0b10000,0b11110,0b00001,0b00001,0b10001,0b01110],
        '6' => [0b00110,0b01000,0b10000,0b11110,0b10001,0b10001,0b01110],
        '7' => [0b11111,0b00001,0b00010,0b00100,0b01000,0b01000,0b01000],
        '8' => [0b01110,0b10001,0b10001,0b01110,0b10001,0b10001,0b01110],
        '9' => [0b01110,0b10001,0b10001,0b01111,0b00001,0b00010,0b01100],

        'A' => [0b00100,0b01010,0b10001,0b11111,0b10001,0b10001,0b10001],
        'B' => [0b11110,0b10001,0b10001,0b11110,0b10001,0b10001,0b11110],
        'C' => [0b01110,0b10001,0b10000,0b10000,0b10000,0b10001,0b01110],
        'D' => [0b11110,0b10001,0b10001,0b10001,0b10001,0b10001,0b11110],
        'E' => [0b11111,0b10000,0b10000,0b11110,0b10000,0b10000,0b11111],
        'F' => [0b11111,0b10000,0b10000,0b11110,0b10000,0b10000,0b10000],
        'G' => [0b01110,0b10001,0b10000,0b10111,0b10001,0b10001,0b01110],
        'H' => [0b10001,0b10001,0b10001,0b11111,0b10001,0b10001,0b10001],
        'I' => [0b01110,0b00100,0b00100,0b00100,0b00100,0b00100,0b01110],
        'J' => [0b00111,0b00010,0b00010,0b00010,0b10010,0b10010,0b01100],
        'K' => [0b10001,0b10010,0b10100,0b11000,0b10100,0b10010,0b10001],
        'L' => [0b10000,0b10000,0b10000,0b10000,0b10000,0b10000,0b11111],
        'M' => [0b10001,0b11011,0b10101,0b10101,0b10001,0b10001,0b10001],
        'N' => [0b10001,0b11001,0b10101,0b10011,0b10001,0b10001,0b10001],
        'O' => [0b01110,0b10001,0b10001,0b10001,0b10001,0b10001,0b01110],
        'P' => [0b11110,0b10001,0b10001,0b11110,0b10000,0b10000,0b10000],
        'Q' => [0b01110,0b10001,0b10001,0b10001,0b10101,0b10010,0b01101],
        'R' => [0b11110,0b10001,0b10001,0b11110,0b10100,0b10010,0b10001],
        'S' => [0b01111,0b10000,0b10000,0b01110,0b00001,0b00001,0b11110],
        'T' => [0b11111,0b00100,0b00100,0b00100,0b00100,0b00100,0b00100],
        'U' => [0b10001,0b10001,0b10001,0b10001,0b10001,0b10001,0b01110],
        'V' => [0b10001,0b10001,0b10001,0b10001,0b01010,0b01010,0b00100],
        'W' => [0b10001,0b10001,0b10101,0b10101,0b10101,0b11011,0b10001],
        'X' => [0b10001,0b10001,0b01010,0b00100,0b01010,0b10001,0b10001],
        'Y' => [0b10001,0b10001,0b01010,0b00100,0b00100,0b00100,0b00100],
        'Z' => [0b11111,0b00001,0b00010,0b00100,0b01000,0b10000,0b11111],

        'a' => [0,0,0b01110,0b00001,0b01111,0b10001,0b01111],
        'b' => [0b10000,0b10000,0b11110,0b10001,0b10001,0b10001,0b11110],
        'c' => [0,0,0b01110,0b10000,0b10000,0b10000,0b01110],
        'd' => [0b00001,0b00001,0b01111,0b10001,0b10001,0b10001,0b01111],
        'e' => [0,0,0b01110,0b10001,0b11111,0b10000,0b01110],
        'f' => [0b00110,0b01001,0b01000,0b11100,0b01000,0b01000,0b01000],
        'g' => [0,0,0b01111,0b10001,0b10001,0b01111,0b00001,],
        'h' => [0b10000,0b10000,0b11110,0b10001,0b10001,0b10001,0b10001],
        'i' => [0b00100,0,0b01100,0b00100,0b00100,0b00100,0b01110],
        'j' => [0b00010,0,0b00110,0b00010,0b00010,0b10010,0b01100],
        'k' => [0b10000,0b10010,0b10100,0b11000,0b10100,0b10010,0b10001],
        'l' => [0b01100,0b00100,0b00100,0b00100,0b00100,0b00100,0b01110],
        'm' => [0,0,0b11010,0b10101,0b10101,0b10101,0b10101],
        'n' => [0,0,0b11110,0b10001,0b10001,0b10001,0b10001],
        'o' => [0,0,0b01110,0b10001,0b10001,0b10001,0b01110],
        'p' => [0,0,0b11110,0b10001,0b10001,0b11110,0b10000],
        'q' => [0,0,0b01111,0b10001,0b10001,0b01111,0b00001],
        'r' => [0,0,0b10110,0b11001,0b10000,0b10000,0b10000],
        's' => [0,0,0b01111,0b10000,0b01110,0b00001,0b11110],
        't' => [0b01000,0b01000,0b11100,0b01000,0b01000,0b01001,0b00110],
        'u' => [0,0,0b10001,0b10001,0b10001,0b10011,0b01101],
        'v' => [0,0,0b10001,0b10001,0b01010,0b01010,0b00100],
        'w' => [0,0,0b10001,0b10101,0b10101,0b10101,0b01010],
        'x' => [0,0,0b10001,0b01010,0b00100,0b01010,0b10001],
        'y' => [0,0,0b10001,0b10001,0b01111,0b00001,0b01110],
        'z' => [0,0,0b11111,0b00010,0b00100,0b01000,0b11111],

        '(' => [0b00010,0b00100,0b01000,0b01000,0b01000,0b00100,0b00010],
        ')' => [0b01000,0b00100,0b00010,0b00010,0b00010,0b00100,0b01000],
        '[' => [0b00110,0b00100,0b00100,0b00100,0b00100,0b00100,0b00110],
        ']' => [0b01100,0b00100,0b00100,0b00100,0b00100,0b00100,0b01100],
        '{' => [0b00010,0b00100,0b00100,0b01000,0b00100,0b00100,0b00010],
        '}' => [0b01000,0b00100,0b00100,0b00010,0b00100,0b00100,0b01000],
        '<' => [0b00010,0b00100,0b01000,0b01000,0b01000,0b00100,0b00010],
        '>' => [0b01000,0b00100,0b00010,0b00010,0b00010,0b00100,0b01000],
        '+' => [0,0b00100,0b00100,0b11111,0b00100,0b00100,0],
        '*' => [0,0b10101,0b01110,0b11111,0b01110,0b10101,0],
        '=' => [0,0b11111,0,0b11111,0,0,0],
        '?' => [0b01110,0b10001,0b00010,0b00100,0b00100,0,0b00100],
        '\''=> [0b00100,0b00100,0,0,0,0,0],
        '"' => [0b01010,0b01010,0,0,0,0,0],
        '|' => [0b00100,0b00100,0b00100,0b00100,0b00100,0b00100,0b00100],
        '@' => [0b01110,0b10001,0b00001,0b01101,0b10101,0b10101,0b01110],
        '#' => [0b01010,0b11111,0b01010,0b01010,0b11111,0b01010,0b01010],
        '%' => [0b11001,0b11010,0b00100,0b01000,0b10110,0b00110,0],
        '^' => [0b00100,0b01010,0b10001,0,0,0,0],
        '~' => [0,0b01101,0b10110,0,0,0,0],
        '`' => [0b01000,0b00100,0,0,0,0,0],

        _ => glyph_5x7('?'),
    }
}

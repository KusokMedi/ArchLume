//! A tiny, self-contained 5x7 bitmap font.
//!
//! First-party and dependency-free (no external font assets, so no third-party
//! font licence to track). Each glyph is seven rows of five bits, most
//! significant bit = leftmost pixel. Enough for the panel's early text
//! (clock digits, uppercase labels); a proper vector-font stack can replace it
//! later without changing callers.

/// A mutable BGRA pixel buffer plus its dimensions.
pub struct Canvas<'a> {
    pub data: &'a mut [u8],
    pub width: u32,
    pub height: u32,
}

impl Canvas<'_> {
    /// Fills the whole canvas with a single colour.
    pub fn fill(&mut self, color: [u8; 3]) {
        let [r, g, b] = color;
        for pixel in self.data.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[b, g, r, 0xFF]);
        }
    }

    /// Draws `text` at `(x, y)`, scaled by `scale`. Pixels outside the canvas
    /// are clipped.
    pub fn draw_text(&mut self, x: i32, y: i32, scale: u32, text: &str, color: [u8; 3]) {
        let mut cursor = x;
        for ch in text.chars() {
            if let Some(rows) = glyph(ch.to_ascii_uppercase()) {
                for (row, bits) in rows.iter().enumerate() {
                    for col in 0..5u32 {
                        if bits & (1 << (4 - col)) == 0 {
                            continue;
                        }
                        self.fill_block(
                            cursor + (col * scale) as i32,
                            y + (row as u32 * scale) as i32,
                            scale,
                            color,
                        );
                    }
                }
            }
            cursor += (6 * scale) as i32;
        }
    }

    fn fill_block(&mut self, x: i32, y: i32, scale: u32, color: [u8; 3]) {
        let [r, g, b] = color;
        for dy in 0..scale as i32 {
            let py = y + dy;
            if py < 0 || py >= self.height as i32 {
                continue;
            }
            for dx in 0..scale as i32 {
                let px = x + dx;
                if px < 0 || px >= self.width as i32 {
                    continue;
                }
                let idx = ((py as u32 * self.width + px as u32) * 4) as usize;
                self.data[idx] = b;
                self.data[idx + 1] = g;
                self.data[idx + 2] = r;
                self.data[idx + 3] = 0xFF;
            }
        }
    }
}

/// Width in pixels that `text` occupies at the given scale.
pub fn text_width(text: &str, scale: u32) -> u32 {
    text.chars().count() as u32 * 6 * scale
}

fn glyph(ch: char) -> Option<[u8; 7]> {
    Some(match ch {
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        ':' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00110,
        ],
        ' ' => [0, 0, 0, 0, 0, 0, 0],
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_advance_including_spacing() {
        assert_eq!(text_width("01", 2), 24);
        assert_eq!(text_width("", 2), 0);
    }

    #[test]
    fn draws_pixels_and_clips() {
        let (w, h) = (16u32, 8u32);
        let mut data = vec![0u8; (w * h * 4) as usize];
        let mut canvas = Canvas {
            data: &mut data,
            width: w,
            height: h,
        };
        canvas.draw_text(0, 0, 1, "1", [255, 255, 255]);
        let lit = canvas.data.chunks_exact(4).filter(|p| p[3] == 0xFF).count();
        assert!(lit > 0);

        // Fully off-canvas must not panic or write.
        let mut empty = vec![0u8; (w * h * 4) as usize];
        let mut canvas = Canvas {
            data: &mut empty,
            width: w,
            height: h,
        };
        canvas.draw_text(1000, 1000, 1, "9", [255, 255, 255]);
        assert!(canvas.data.iter().all(|b| *b == 0));
    }

    #[test]
    fn fill_sets_opaque_bgra() {
        let (w, h) = (2u32, 2u32);
        let mut data = vec![0u8; (w * h * 4) as usize];
        let mut canvas = Canvas {
            data: &mut data,
            width: w,
            height: h,
        };
        canvas.fill([0x1E, 0x88, 0xE5]);
        assert_eq!(&data[0..4], &[0xE5, 0x88, 0x1E, 0xFF]);
    }

    #[test]
    fn unknown_glyphs_still_advance() {
        assert_eq!(text_width("~", 1), 6);
    }
}

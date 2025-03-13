pub struct ColorValue {
    pub name: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorValue {
    pub fn from_hex(name: &str, hex: &str) -> Self {
        let (r, g, b) = ColorValue::hex_to_rgb(hex);

        ColorValue {
            name: name.to_string(),
            r,
            g,
            b,
        }
    }

    pub fn add_brightness(&mut self, brightness: i32) {
        self.r = (self.r as i32 + brightness).clamp(0, 255) as u8;
        self.g = (self.g as i32 + brightness).clamp(0, 255) as u8;
        self.b = (self.b as i32 + brightness).clamp(0, 255) as u8;
    }

    pub fn set_brightness(&mut self, brightness: u8) {
        let current_brightness = ((self.r + self.g + self.b) as f32 / 3.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        let brightness_diff = brightness as f32 / current_brightness as f32;

        self.r = (self.r as f32 * brightness_diff).clamp(0.0, 255.0) as u8;
        self.g = (self.g as f32 * brightness_diff).clamp(0.0, 255.0) as u8;
        self.b = (self.b as f32 * brightness_diff).clamp(0.0, 255.0) as u8;
    }

    pub fn invert(&mut self) {
        self.r = 255 - self.r;
        self.g = 255 - self.g;
        self.b = 255 - self.b;
    }

    pub fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        if let Ok(rgb) = u32::from_str_radix(hex, 16) {
            let r = (rgb >> 16) as u8;
            let g = (rgb >> 8 & 0xFF) as u8;
            let b = (rgb & 0xFF) as u8;

            (r, g, b)
        } else {
            (0, 0, 0)
        }
    }

    pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
        format!("{:02X}{:02X}{:02X}", r, g, b)
    }

    pub fn set_value_from_hex(&mut self, hex: &str) {
        let (r, g, b) = ColorValue::hex_to_rgb(hex);
        self.r = r;
        self.g = g;
        self.b = b;
    }

    pub fn set_value_from_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.r = r;
        self.g = g;
        self.b = b;
    }

    pub fn hex(&self) -> String {
        ColorValue::rgb_to_hex(self.r, self.g, self.b)
    }
}

#[derive(Clone)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone)]
pub struct HEX {
    pub value: String,
}

#[derive(Clone)]
pub enum Color {
    RGB(RGB),
    HEX(HEX),
}

use super::color::{IntoHSV, IntoRGB};
use super::hsv::HSV;
use super::rgb::RGB;

#[derive(Clone)]
pub struct HEX(pub String);

impl HEX {
    pub fn new(value: String) -> Self {
        HEX(value)
    }
}

impl IntoRGB for HEX {
    fn to_rgb(&self) -> RGB {
        if let Ok(rgb) = u32::from_str_radix(&self.0, 16) {
            let r = (rgb >> 16) as u8;
            let g = (rgb >> 8 & 0xFF) as u8;
            let b = (rgb & 0xFF) as u8;

            RGB::new(r, g, b)
        } else {
            RGB::new(0, 0, 0)
        }
    }
}

impl IntoHSV for HEX {
    fn to_hsv(&self) -> HSV {
        self.to_rgb().to_hsv()
    }
}

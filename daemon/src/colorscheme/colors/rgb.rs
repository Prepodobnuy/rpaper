use super::color::{IntoHEX, IntoHSV};
use super::hex::HEX;
use super::hsv::HSV;

#[derive(Clone)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        RGB { r, g, b }
    }
}

impl IntoHEX for RGB {
    fn to_hex(&self) -> super::hex::HEX {
        HEX::new(format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b))
    }
}

impl IntoHSV for RGB {
    fn to_hsv(&self) -> super::hsv::HSV {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));

        let v = max;
        let s = if max == 0.0 { 0.0 } else { (max - min) / max };

        let h = if max == min {
            0.0
        } else if max == r {
            60.0 * ((g - b) / (max - min))
        } else if max == g {
            60.0 * (2.0 + (b - r) / (max - min))
        } else {
            60.0 * (4.0 + (r - g) / (max - min))
        };

        HSV::new(
            h.clamp(0.0, 360.0) as u16,
            (s * 100.0) as u16,
            (v * 100.0) as u16,
        )
    }
}

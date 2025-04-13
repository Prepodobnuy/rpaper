use super::color::{IntoHEX, IntoRGB};
use super::hex::HEX;
use super::rgb::RGB;

#[derive(Clone)]
pub struct HSV {
    pub h: u16,
    pub s: u16,
    pub v: u16,
}

impl HSV {
    pub fn new(h: u16, s: u16, v: u16) -> Self {
        HSV { h, s, v }
    }

    // TODO use this func
    pub fn _normalize(&self) -> Self {
        HSV {
            h: self.h.clamp(0, 360),
            s: self.s.clamp(0, 100),
            v: self.v.clamp(0, 100),
        }
    }
}

impl IntoRGB for HSV {
    fn to_rgb(&self) -> RGB {
        let h = self.h as f32 / 360.0;
        let s = self.s as f32 / 100.0;
        let v = self.v as f32 / 100.0;

        let i = (h * 6.0).floor();
        let f = h * 6.0 - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);

        let (r, g, b) = match (i % 6.0) as u8 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            5 => (v, p, q),
            _ => unreachable!(),
        };

        RGB::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

impl IntoHEX for HSV {
    fn to_hex(&self) -> HEX {
        self.to_rgb().to_hex()
    }
}

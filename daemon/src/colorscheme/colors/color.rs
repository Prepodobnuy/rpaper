use super::hex::HEX;
use super::hsv::HSV;
use super::rgb::RGB;

#[derive(Clone)]
pub enum Color {
    RGB(RGB),
    HEX(HEX),
    // TODO implement Hsv() template function
    _HSV(HSV),
}

pub trait IntoRGB {
    fn to_rgb(&self) -> RGB;
}

pub trait IntoHEX {
    fn to_hex(&self) -> HEX;
}

pub trait IntoHSV {
    fn to_hsv(&self) -> HSV;
}

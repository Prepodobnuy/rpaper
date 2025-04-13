use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum OrderBy {
    Hue,
    Saturation,
    Brightness,
    Semantic,
}

impl FromStr for OrderBy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "h" | "H" | "hue" => {Ok(OrderBy::Hue)},
            "s" | "S" | "saturation" => {Ok(OrderBy::Saturation)},
            "v" | "b" | "V" | "B" | "brightness" => {Ok(OrderBy::Brightness)},
            "sem" | "semantic" => {Ok(OrderBy::Semantic)},
            _ => {Err(String::new())},
        }
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub struct RwalParams {
    pub thumb_range: (u32, u32),
    pub clamp_range: (f32, f32),
    pub accent_color: u32,
    pub colors: u32,
    pub order: OrderBy,
}

impl RwalParams {
    pub fn new(
        thumb_range: (u32, u32),
        clamp_range: (f32, f32),
        accent_color: u32,
        colors: u32,
        order: OrderBy,
    ) -> Self {
        RwalParams {
            thumb_range,
            clamp_range,
            accent_color,
            colors,
            order,
        }
    }
}

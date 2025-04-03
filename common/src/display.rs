use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Serialize, Deserialize)]
pub struct Display {
    pub name: String,
    pub w: u32,
    pub h: u32,
    pub x: u32,
    pub y: u32,
}

impl Display {
    pub fn new(name: String, w: u32, h: u32, x: u32, y: u32) -> Self {
        Display { name, w, h, x, y }
    }
}

impl FromStr for Display {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut name: String = String::new();
        let mut w: u32 = 0;
        let mut h: u32 = 0;
        let mut x: u32 = 0;
        let mut y: u32 = 0;

        if s.split(":").collect::<Vec<_>>().len() != 5 {
            return Err(String::from("Unable to parse string"));
        }

        s.split(":").enumerate().for_each(|(i, param)| match i {
            0 => name = String::from(param),
            1 => w = param.parse().unwrap_or(0),
            2 => h = param.parse().unwrap_or(0),
            3 => x = param.parse().unwrap_or(0),
            4 => y = param.parse().unwrap_or(0),
            _ => {}
        });

        Ok(Display::new(name, w, h, x, y))
    }
}


use std::fs::File;
use std::io::Read;

use serde_json::Value;

use crate::colorscheme::rwal::OrderBy;
use crate::expand_user;
use crate::wallpaper::display::Display;
use crate::wallpaper::display::ImageOperations;
use crate::colorscheme::template::Template;
use crate::colorscheme::rwal::RwalParams;
use crate::wallpaper::display::WCacheInfo;


#[derive(Clone)]
pub struct Config {
    pub displays: Option<Vec<Display>>,
    pub templates: Option<Vec<Template>>,
    pub wallpaper_set_command: Option<String>,
    pub resize_algorithm: Option<String>,
    pub rwal_params: Option<RwalParams>,
    pub image_operations: Option<ImageOperations>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            displays: None,
            templates: None,
            wallpaper_set_command: None,
            resize_algorithm: None,
            rwal_params: None,
            image_operations: None,
        }
    }

    pub fn read(&mut self, path: &str) {
        if let Some(value) = read_value(path) {
            self.displays = read_displays(&value);
            self.templates = read_templates(&value);
            self.wallpaper_set_command = read_wallpaper_set_command(&value);
            self.resize_algorithm = read_resize_algorithm(&value);
            self.rwal_params = read_rwal_params(&value);
            self.image_operations = read_image_operations(&value);
        }
    }

    pub fn read_from_string(&mut self, string: String) {
        if let Some(value) = read_value_from_string(string) {
            self.displays = read_displays(&value);
            self.templates = read_templates(&value);
            self.wallpaper_set_command = read_wallpaper_set_command(&value);
            self.resize_algorithm = read_resize_algorithm(&value);
            self.rwal_params = read_rwal_params(&value);
            self.image_operations = read_image_operations(&value);
        }
    }
}

fn read_value(path: &str) -> Option<Value> {
    if let Ok(mut file) = File::open(path) {
        let mut json_data = String::new();
        file.read_to_string(&mut json_data).unwrap();

        if let Ok(data) = serde_json::from_str(&json_data) {
            return data;
        }
    }
    None
}

fn read_value_from_string(string: String) -> Option<Value> {
    if let Ok(data) = serde_json::from_str(&string) {
        return data;
    }
    None
}

fn read_displays(value: &Value) -> Option<Vec<Display>> {
    let mut displays = Vec::new();

    for raw_display in value["displays"].as_array().unwrap() {
        let name = String::from(raw_display["name"].as_str().unwrap());
        let w = raw_display["w"].as_u64().unwrap() as u32;
        let h = raw_display["h"].as_u64().unwrap() as u32;
        let x = raw_display["x"].as_u64().unwrap() as u32;
        let y = raw_display["y"].as_u64().unwrap() as u32;

        displays.push(Display::new(w, h, x, y, name))
    }

    Some(displays)
}

fn read_templates(value: &Value) -> Option<Vec<Template>> {
    let mut templates = Vec::new();

    for template_path in value["templates"].as_array().unwrap() {
        if let Some(path) = template_path.as_str() {
            templates.push(Template::new(&expand_user(path)));
        }
    }

    Some(templates)
}

fn read_wallpaper_set_command(value: &Value) -> Option<String> {
    if let Some(command) = value["wall_command"].as_str() {
        return Some(String::from(command));
    }

    None
}

fn read_resize_algorithm(value: &Value) -> Option<String> {
    if let Some(command) = value["resize_algorithm"].as_str() {
        return Some(String::from(command));
    }

    None
}

fn read_rwal_params(value: &Value) -> Option<RwalParams> {
    let rwal = value.get("rwal")?;

    let thumb_w = rwal["thumb_w"].as_u64().unwrap_or(200) as u32;
    let thumb_h = rwal["thumb_h"].as_u64().unwrap_or(200) as u32;
    let thumb_range = (thumb_w, thumb_h);

    let clamp_min = rwal["clamp_min"].as_f64().unwrap_or(140.0) as f32;
    let clamp_max = rwal["clamp_max"].as_f64().unwrap_or(170.0) as f32;
    let clamp_range = (clamp_min, clamp_max);
    
    let accent_color = rwal["accent_color"].as_u64().unwrap_or(4) as u32;
    let colors = rwal["rwal_colors"].as_u64().unwrap_or(7) as u32;

    let order = match rwal["order_by"].as_str().unwrap_or("h") {
        "s" | "S" => {OrderBy::Saturation},
        "v" | "V" | "b" | "B" => {OrderBy::Brightness},
        _ => {OrderBy::Hue},
    };

    Some(RwalParams::new(thumb_range, clamp_range, accent_color, colors, order))
}

fn read_image_operations(value: &Value) -> Option<ImageOperations> {
    let impg = value.get("impg")?;

    let contrast = impg["contrast"].as_f64().unwrap_or(0.0) as f32;
    let brightness = impg["brightness"].as_i64().unwrap_or(0) as i32;
    let hue = impg["huerotate"].as_i64().unwrap_or(0) as i32;
    let blur = impg["blur"].as_f64().unwrap_or(0.0) as f32;
    let invert = impg["invert"].as_bool().unwrap_or(false);
    let flip_h = impg["flip_h"].as_bool().unwrap_or(false);
    let flip_v = impg["flip_v"].as_bool().unwrap_or(false);

    Some(ImageOperations::new(contrast, brightness, hue, blur, invert, flip_h, flip_v))
}

pub trait JsonString {
    fn json(& self) -> String {
        String::new()
    }
}

impl JsonString for Display {
    fn json(& self) -> String {
        format!(
            "{{\"name\":\"{}\",\"w\":{},\"h\":{},\"x\":{},\"y\":{}}}",
            self.name(),
            self.width(),
            self.height(),
            self.x(),
            self.y(),
        )
    }
}

impl JsonString for Vec<Display> {
    fn json(& self) -> String {
        let json_strings: Vec<String> = self.iter()
            .map(|d| d.json())
            .collect();

        format!("[{}]", json_strings.join(","))
    }
}

impl JsonString for Template {
    fn json(& self) -> String {
        format!(
            "\"{}\"",
            self.path
        )
    }
}

impl JsonString for Vec<Template> {
    fn json(& self) -> String {
        let json_strings: Vec<String> = self.into_iter()
            .map(|t| t.json())
            .collect();

        format!("[{}]", json_strings.join(","))
    }
}

impl JsonString for ImageOperations {
    fn json(& self) -> String {
        format!(
            "{{\"contrast\":{},\"brightness\":{},\"huerotate\":{},\"blur\":{},\"invert\":{},\"flip_h\":{},\"flip_v\":{}}}",
            self.contrast,
            self.brightness,
            self.hue,
            self.blur,
            self.invert,
            self.flip_h,
            self.flip_v
        )
    }
}

impl JsonString for RwalParams {
    fn json(& self) -> String {
        format!(
            "{{\"thumb_w\":{},\"thumb_h\":{},\"accent_color\":{},\"clamp_min\":{},\"clamp_max\":{}}}",
            self.thumb_range.0,
            self.thumb_range.1,
            self.accent_color,
            self.clamp_range.0,
            self.clamp_range.1
        )
    }
}

impl JsonString for Config {
    fn json(& self) -> String {
        format!(
            "{{\"displays\":{},\"templates\":{},\"impg\":{},\"rwal\":{}}}",
            {
                if let Some(displays) = &self.displays {
                    displays.json()
                } else {
                    "[]".to_string()
                }
            },
            {
                if let Some(templates) = &self.templates {
                    templates.json()
                } else {
                    "[]".to_string()
                }
            },
            {
                if let Some(image_operations) = &self.image_operations {
                    image_operations.json()
                } else {
                    "{}".to_string()
                }
            },
            {
                if let Some(rwal_params) = &self.rwal_params {
                    rwal_params.json()
                } else {
                    "{}".to_string()
                }
            }
        )
    }
}

impl JsonString for WCacheInfo {
    fn json(& self) -> String {
        format!(
            "{{\"display\":\"{}\",\"path\":\"{}\"}}",
            self.display_name,
            self.path
        )
    }
}
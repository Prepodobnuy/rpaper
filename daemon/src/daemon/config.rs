use std::fs::File;
use std::io::Read;
use std::path;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::colorscheme::rwal::rwal_params::OrderBy;
use crate::colorscheme::rwal::rwal_params::RwalParams;
use crate::expand_user;
use crate::wallpaper::image::ImageOperations;
use common::display::Display;

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub displays: Option<Vec<Display>>,
    pub templates: Option<Vec<String>>,
    pub set_command: Option<String>,
    pub resize_algorithm: Option<String>,
    pub last_call_file: Option<String>,
    pub rwal_params: Option<RwalParams>,
    pub image_operations: Option<ImageOperations>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            displays: None,
            templates: None,
            set_command: None,
            resize_algorithm: None,
            last_call_file: None,
            rwal_params: None,
            image_operations: None,
        }
    }

    pub fn read(&mut self, path: &str) {
        if let Some(value) = read_value(path) {
            self.displays = read_displays(&value);
            self.templates = read_templates(&value);
            self.set_command = read_wallpaper_set_command(&value);
            self.resize_algorithm = read_resize_algorithm(&value);
            self.last_call_file = read_last_call_file(&value);
            self.rwal_params = read_rwal_params(&value);
            self.image_operations = read_image_operations(&value);
        }
    }

    pub fn read_from_string(&mut self, string: String) {
        if let Some(value) = read_value_from_string(string) {
            self.displays = read_displays(&value);
            self.templates = read_templates(&value);
            self.set_command = read_wallpaper_set_command(&value);
            self.resize_algorithm = read_resize_algorithm(&value);
            self.last_call_file = read_last_call_file(&value);
            self.rwal_params = read_rwal_params(&value);
            self.image_operations = read_image_operations(&value);
        }
    }
}

fn read_value(path: &str) -> Option<Value> {
    if let Ok(mut file) = File::open(path) {
        let mut json_data = String::new();

        if let Err(_) = file.read_to_string(&mut json_data) {
            return None;
        }

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

    for raw_display in value["displays"].as_array()? {
        let name = raw_display["name"].as_str()?.to_string();
        let w = raw_display["w"].as_u64()? as u32;
        let h = raw_display["h"].as_u64()? as u32;
        let x = raw_display["x"].as_u64()? as u32;
        let y = raw_display["y"].as_u64()? as u32;

        displays.push(Display::new(name, w, h, x, y))
    }

    Some(displays)
}

fn read_templates(value: &Value) -> Option<Vec<String>> {
    let mut templates = Vec::new();

    for template_path in value["templates"].as_array()? {
        if let Some(path) = template_path.as_str() {
            if path::Path::new(&expand_user(path)).exists() {
                templates.push(expand_user(path));
            }
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

fn read_last_call_file(value: &Value) -> Option<String> {
    if let Some(path) = value["last_call_file"].as_str() {
        return Some(expand_user(path));
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
        "s" | "S" => OrderBy::Saturation,
        "v" | "V" | "b" | "B" => OrderBy::Brightness,
        "sem" | "semantic" => OrderBy::Semantic,
        _ => OrderBy::Hue,
    };

    Some(RwalParams::new(
        thumb_range,
        clamp_range,
        accent_color,
        colors,
        order,
    ))
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

    Some(ImageOperations::new(
        contrast, brightness, hue, blur, invert, flip_h, flip_v,
    ))
}


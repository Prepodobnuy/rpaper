use std::io::Write;
use std::path::Path;
use std::fs::{self, File};

use crate::{expand_user, spawn, system};

#[derive(Clone)]
pub struct Template {
    pub path: String,
}

enum TemplatePart {
    Params,
    Colors,
    Caption,
    None,
}

impl Template {
    pub fn new(path: &str) -> Self {
        Template {
            path: String::from(path)
        }
    }

    pub fn process(& self, colors: Vec<String>) {
        if !Path::new(&self.path).exists() {
            return;
        }

        let mut template_part = TemplatePart::None;
        
        let mut params: Vec<String> = Vec::new();
        let mut color_vars: Vec<String> = Vec::new();
        let mut caption: Vec<String> = Vec::new();

        if let Ok(raw_template) = fs::read_to_string(&self.path) {
            for line in raw_template.lines() {
                match template_part {
                    TemplatePart::Params => {
                        if line.is_empty() || line.starts_with("#") {
                            continue;
                        }

                        match line.trim() {
                            "[colors]" => {
                                template_part = TemplatePart::Colors
                            },
                            "[caption]" => {
                                template_part = TemplatePart::Caption
                            },
                            _ => {
                                params.push(line.trim().to_string())
                            },
                        }
                    },
                    TemplatePart::Colors => {
                        if line.is_empty() || line.starts_with("#") {
                            continue;
                        }

                        match line.trim() {
                            "[param]" => {
                                template_part = TemplatePart::Params
                            },
                            "[caption]" => {
                                template_part = TemplatePart::Caption
                            },
                            _ => {
                                color_vars.push(line.trim().to_string())
                            },
                        }
                    },
                    TemplatePart::Caption => {
                        caption.push(line.to_string())
                    },
                    TemplatePart::None => {
                        if line.is_empty() {
                            continue;
                        }
                        match line.trim() {
                            "[param]" => {
                                template_part = TemplatePart::Params
                            },
                            "[colors]" => {
                                template_part = TemplatePart::Colors
                            },
                            "[caption]" => {
                                template_part = TemplatePart::Caption
                            },
                            _ => {
                                continue;
                            },
                        }
                    },
                }
            }
        }

        let params_val: Params;
        let colors_val: Vec<Color>;
        let mut caption: String = get_caption(caption);

        if let Some(val) = get_params(params) {
            params_val = val
        } 
        else {
            return;
        }

        if let Some(val) = get_colors(color_vars) {
            colors_val = val
        }
        else {
            return;
        }

        if let Some(command) = &params_val.before {
            system(command);
        }

        let mut color_values = Vec::new();

        for color in colors_val {
            let col;
            if color.index >= colors.len() {
                col = "000000"
            } else {
                col = &colors[color.index]
            }

            if color.name.contains("{br}") {
                for i in 1..20 {
                    let mut _lighter = ColorValue::from_hex(
                        &color.name.replace("{br}", &format!("LR{}", i)),
                        &col,
                        (10 * i) + color.change,
                    );
                    let mut _darker = ColorValue::from_hex(
                        &color.name.replace("{br}", &format!("DR{}", i)),
                        &col,
                        (-10 * i) + color.change,
                    );

                    if color.inversed {
                        _lighter.inverse();
                        _darker.inverse();
                    }

                    color_values.push(_lighter);
                    color_values.push(_darker);
                }
                let mut _color_value = ColorValue::from_hex(
                    &color.name.replace("{br}", ""),
                    &col,
                    color.change,
                );

                if color.inversed {
                    _color_value.inverse();
                }

                color_values.push(_color_value);
            }
        }

        for color_value in color_values {
            caption = color_value.apply(&params_val.format, caption)
        }

        if let Ok(mut file) = File::create(expand_user(&params_val.path)) {
            let _ = file.write_all(caption.as_bytes());
        }

        if let Some(command) = &params_val.after {
            spawn(command);
        }

    }
}

struct Params {
    path: String,
    format: String,
    before: Option<String>,
    after: Option<String>,
}

struct Color {
    name: String,
    index: usize,
    change: i32,
    inversed: bool,
}

struct ColorValue {
    name: String,
    r: u8,
    g: u8,
    b: u8,
}

impl ColorValue {
    fn from_hex(name: &str, hex: &str, change: i32) -> Self {
        let (r, g, b) = hex_to_rgb(hex);
        let r = r as i32 + change;
        let g = g as i32 + change;
        let b = b as i32 + change;
        
        let r = if r > 255 {255} else {r};
        let r: u8 = if r < 0 {0} else {r as u8};

        let g = if g > 255 {255} else {g};
        let g: u8 = if g < 0 {0} else {g as u8};

        let b = if b > 255 {255} else {b};
        let b: u8 = if b < 0 {0} else {b as u8};

        ColorValue {
            name: name.to_string(),
            r,
            g,
            b,
        }
    }

    fn inverse(&mut self) {
        self.r = 255 - self.r;
        self.g = 255 - self.g;
        self.b = 255 - self.b;
    }

    fn apply(& self, format: &str, message: String) -> String {
        let format = format.replace("{HEX}", &rgb_to_hex(self.r, self.g, self.b))
            .replace("{R}", &self.r.to_string())
            .replace("{G}", &self.g.to_string())
            .replace("{B}", &self.b.to_string());

        message.replace(&self.name, &format)
    }
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.strip_prefix('#').unwrap_or(hex);

    if let Ok(rgb) = u32::from_str_radix(hex, 16) {
        let r = (rgb >> 16) as u8;
        let g = (rgb >> 8 & 0xFF) as u8;
        let b = (rgb & 0xFF) as u8;
    
        (r, g, b)
    } 
    else {
        (0, 0, 0)
    }
}

fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("{:02X}{:02X}{:02X}", r, g, b)
}

fn get_params(input: Vec<String>) -> Option<Params> {
    let mut path = String::new();
    let mut format = String::new();
    let mut before = String::new();
    let mut after = String::new();

    for part in input {
        if let Some(part) = part.strip_prefix("path:") {
            path = part.to_string();
        }
        else if let Some(part) = part.strip_prefix("format:") {
            format = part.to_string();
        }
        else if let Some(part) = part.strip_prefix("before:") {
            before = part.to_string();
        }
        else if let Some(part) = part.strip_prefix("after:") {
            after = part.to_string();
        }
    }

    if path.is_empty() || format.is_empty() {
        return None;
    }

    Some(Params {
        path,
        format,
        before: Some(before),
        after: Some(after),
    })
}

fn get_colors(input: Vec<String>) -> Option<Vec<Color>> {
    let mut colors: Vec<Color> = Vec::new();
    for raw_color in input {
        let splitted: Vec<&str> = raw_color.split(":").collect();

        let name;
        let index;
        let mut change = 0;
        let mut inversed = false;

        match splitted.len() {
            2 => {
                name = splitted[0].to_string();

                index = if let Ok(i) = splitted[1].parse::<usize>() { i } else { 0 };
            },
            3 => {
                name = splitted[0].to_string();

                index = if let Ok(i) = splitted[1].parse::<usize>() { i } else { 0 };

                change = if let Ok(c) = splitted[2].parse::<i32>() { c } else { 0 };
            },
            4 => {
                name = splitted[0].to_string();

                index = if let Ok(i) = splitted[1].parse::<usize>() { i } else { 0 };

                change = if let Ok(c) = splitted[2].parse::<i32>() { c } else { 0 };

                if let Ok(i) = splitted[3].parse::<i32>() {                    
                    inversed = i == 1
                }
            },
            _ => {
                continue;
            },
        }

        colors.push(Color {
            name,
            index,
            change,
            inversed,
        });
    }

    if colors.is_empty() {
        None
    } 
    else {
        Some(colors)
    }
}

fn get_caption(input: Vec<String>) -> String {
    input.join("\n")
}

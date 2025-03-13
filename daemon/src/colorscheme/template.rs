use std::fs;
use std::path;
use std::str::FromStr;

use crate::{expand_user, spawn, system};

enum Section {
    None,
    Params,
    Colors,
    Config,
}

#[derive(Clone)]
struct Color {
    template: String,
    index: u8,
    brightness: i32,
    invert: bool,
    hex: Option<String>,
}

impl Color {
    fn new(
        template: String,
        index: u8,
        brightness: i32,
        invert: bool,
        hex: Option<String>,
    ) -> Self {
        Color {
            template,
            index,
            brightness,
            invert,
            hex,
        }
    }
}

struct ColorValue {
    name: String,
    r: u8,
    g: u8,
    b: u8,
}

impl ColorValue {
    fn from_hex(name: &str, hex: &str) -> Self {
        let (r, g, b) = ColorValue::hex_to_rgb(hex);

        ColorValue {
            name: name.to_string(),
            r,
            g,
            b,
        }
    }

    fn add_brightness(&mut self, brightness: i32) {
        self.r = (self.r as i32 + brightness).clamp(0, 255) as u8;
        self.g = (self.g as i32 + brightness).clamp(0, 255) as u8;
        self.b = (self.b as i32 + brightness).clamp(0, 255) as u8;
    }

    fn set_brightness(&mut self, brightness: u8) {
        let current_brightness = ((self.r + self.g + self.b) as f32 / 3.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        let brightness_diff = brightness as f32 / current_brightness as f32;

        self.r = (self.r as f32 * brightness_diff).clamp(0.0, 255.0) as u8;
        self.g = (self.g as f32 * brightness_diff).clamp(0.0, 255.0) as u8;
        self.b = (self.b as f32 * brightness_diff).clamp(0.0, 255.0) as u8;
    }

    fn invert(&mut self) {
        self.r = 255 - self.r;
        self.g = 255 - self.g;
        self.b = 255 - self.b;
    }

    fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
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

    fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
        format!("{:02X}{:02X}{:02X}", r, g, b)
    }

    fn set_value_from_hex(&mut self, hex: &str) {
        let (r, g, b) = ColorValue::hex_to_rgb(hex);
        self.r = r;
        self.g = g;
        self.b = b;
    }

    fn hex(&self) -> String {
        ColorValue::rgb_to_hex(self.r, self.g, self.b)
    }
}

#[derive(Clone)]
pub struct Template {
    pub self_path: String,
    paste_path: String,
    color_format: String,
    colors: Vec<Color>,
    before_commands: Vec<String>,
    after_commands: Vec<String>,
    config_caption: String,
}

impl Template {
    pub fn new(path: &str) -> Result<Self, String> {
        if !path::Path::new(path).exists() {
            return Err("path does not exist".to_string());
        }

        let mut params_caption: Vec<String> = Vec::new();
        let mut colors_caption: Vec<String> = Vec::new();
        let mut config_caption: Vec<String> = Vec::new();

        if let Ok(file_caption) = fs::read_to_string(path) {
            let mut section = Section::None;

            for line in file_caption.lines() {
                match section {
                    Section::None => {
                        if line.is_empty() || line.starts_with("#") {
                            continue;
                        }
                        match line.trim() {
                            "[params]" => section = Section::Params,
                            "[colors]" => section = Section::Colors,
                            "[config]" => section = Section::Config,
                            _ => {}
                        }
                        continue;
                    }
                    Section::Params => {
                        if line.is_empty() || line.starts_with("#") {
                            continue;
                        }
                        match line.trim() {
                            "[colors]" => section = Section::Colors,
                            "[config]" => section = Section::Config,
                            _ => params_caption.push(line.trim().to_string()),
                        }
                        continue;
                    }
                    Section::Colors => {
                        if line.is_empty() || line.starts_with("#") {
                            continue;
                        }
                        match line.trim() {
                            "[params]" => section = Section::Params,
                            "[config]" => section = Section::Config,
                            _ => colors_caption.push(line.trim().to_string()),
                        }
                        continue;
                    }
                    Section::Config => {
                        config_caption.push(line.trim().to_string());
                    }
                }
            }
        } else {
            return Err("Something gone wrong".to_string());
        }

        params_caption = apply_include(params_caption);
        colors_caption = apply_include(colors_caption);

        let paste_path = expand_user(&collect_command(&params_caption, "Path(", ")"));
        let color_format = collect_command(&params_caption, "Format(", ")");
        let colors = collect_colors(&colors_caption);

        let before_commands = collect_commands(&params_caption, "ExecBefore(", ")");
        let after_commands = collect_commands(&params_caption, "ExecAfter(", ")");

        Ok(Template {
            self_path: path.to_string(),
            paste_path,
            color_format,
            colors,
            before_commands,
            after_commands,
            config_caption: config_caption.join("\n"),
        })
    }

    fn before(&self) {
        for command in &self.before_commands {
            if !command.is_empty() {
                system(command);
            }
        }
    }

    fn after(&self) {
        for command in &self.after_commands {
            if !command.is_empty() {
                spawn(command);
            }
        }
    }

    pub fn apply(&self, hex_colors: Vec<String>) {
        self.before();

        let mut config = self.config_caption.clone();

        let mut color_values: Vec<ColorValue> = Vec::new();
        for color in &self.colors {
            if color.template.contains("{br}") {
                for i in 1..20 {
                    let mut lighter = ColorValue::from_hex(
                        &color.template.replace("{br}", &format!("LR{i}")),
                        &hex_colors[color.index as usize],
                    );
                    let mut darker = ColorValue::from_hex(
                        &color.template.replace("{br}", &format!("DR{i}")),
                        &hex_colors[color.index as usize],
                    );

                    if let Some(hex) = &color.hex {
                        lighter.set_value_from_hex(hex);
                        darker.set_value_from_hex(hex);
                    }

                    if color.invert {
                        lighter.invert();
                        darker.invert();
                    }

                    lighter.add_brightness((i * 10) + color.brightness);
                    darker.add_brightness((i * -10) + color.brightness);

                    color_values.push(lighter);
                    color_values.push(darker);
                }
            }
            let mut color_value = ColorValue::from_hex(
                &color.template.replace("{br}", ""),
                &hex_colors[color.index as usize],
            );

            if let Some(hex) = &color.hex {
                color_value.set_value_from_hex(hex);
            }

            if color.invert {
                color_value.invert()
            }

            color_value.add_brightness(color.brightness);

            color_values.push(color_value);
        }

        for color_value in color_values {
            let mut format = self.color_format.clone();
            format = format.replace("{R}", &color_value.r.to_string());
            format = format.replace("{G}", &color_value.g.to_string());
            format = format.replace("{B}", &color_value.b.to_string());
            format = format.replace("{HEX}", &color_value.hex());

            config = config.replace(&color_value.name, &format);
        }

        let _ = fs::write(&self.paste_path, config);

        self.after();
    }
}

impl FromStr for Template {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Template::new(&expand_user(s))
    }
}

fn apply_include(caption: Vec<String>) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();

    for line in caption {
        let trim_line = line.trim();

        if !trim_line.starts_with("Include(") || !trim_line.ends_with(")") {
            res.push(line);
            continue;
        }

        if let Ok(caption) = fs::read_to_string(expand_user(&trim_line[8..trim_line.len() - 1])) {
            res.extend(apply_include(
                caption
                    .lines()
                    .map(|l| l.to_string())
                    .collect::<Vec<String>>(),
            ));
        }
    }

    res
}

fn collect_commands(caption: &Vec<String>, prefix: &str, suffix: &str) -> Vec<String> {
    let mut res = Vec::new();
    let prefix_len = prefix.len();

    for line in caption {
        let trim_line = line.trim();

        if !trim_line.starts_with(prefix) || !trim_line.ends_with(suffix) {
            continue;
        }

        let content = &line[prefix_len..trim_line.len() - suffix.len()];
        res.push(content.to_string());
    }

    res
}

fn collect_command(caption: &Vec<String>, prefix: &str, suffix: &str) -> String {
    let mut res = String::new();
    let prefix_len = prefix.len();

    for line in caption {
        let trim_line = line.trim();

        if !trim_line.starts_with(prefix) || !trim_line.ends_with(suffix) {
            continue;
        }

        let content = &line[prefix_len..trim_line.len() - suffix.len()];
        res = content.to_string();
    }

    res
}

fn collect_colors(caption: &Vec<String>) -> Vec<Color> {
    let mut res = Vec::new();

    for command in collect_commands(caption, "Color(", ")") {
        let arguments: Vec<&str> = command.split(",").collect();

        if !arguments.len() < 2 {
            continue;
        }

        let template = arguments[0].trim().to_string();
        let index = arguments[1].trim().parse::<u8>().unwrap_or(0).clamp(0, 15);
        let mut brightness = 0;
        let mut invert = false;

        if arguments.len() > 2 {
            brightness = arguments[2].trim().parse().unwrap_or(0);
        }
        if arguments.len() > 3 {
            invert = matches!(arguments[3].trim(), "1" | "true" | "True");
        }

        res.push(Color::new(template, index, brightness, invert, None))
    }

    for command in collect_commands(caption, "HEX(", ")") {
        let arguments: Vec<&str> = command.split(",").collect();

        if arguments.len() != 2 {
            continue;
        }

        let template = arguments[0].trim().to_string();
        let index: u8 = 0;
        let hex = arguments[1].trim().to_string();
        let brightness = 0;
        let invert = false;

        res.push(Color::new(template, index, brightness, invert, Some(hex)))
    }

    res
}

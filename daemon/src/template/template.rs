use std::fs;
use std::path;
use std::str::FromStr;

use crate::colorscheme::colors::Color;
use crate::colorscheme::colors::ColorValue;
use crate::{expand_user, spawn, system};

use super::parser::collect_colors;
use super::parser::collect_command;
use super::parser::collect_commands;
use super::parser::parse_template;
use super::tags::{EXEC_AFTER_TAG, EXEC_BEFORE_TAG, FORMAT_TAG, PATH_TAG};

#[derive(Clone)]
pub struct Template {
    pub self_path: String,
    conf_path: String,
    conf_caption: String,
    color_format: String,
    color_vars: Vec<ColorVariable>,
    commands_before: Vec<String>,
    commands_after: Vec<String>,
}

impl Template {
    pub fn new(path: &str) -> Result<Self, String> {
        if !path::Path::new(path).exists() {
            return Err(format!("Path '{}' does not exist.", path));
        }

        let mut params_caption: Vec<String> = Vec::new();
        let mut config_caption: String = String::new();

        if let Ok(raw_template) = fs::read_to_string(path) {
            (params_caption, config_caption) = parse_template(raw_template);
        }

        let conf_path = collect_command(&params_caption, PATH_TAG, ")");
        let color_format = collect_command(&params_caption, FORMAT_TAG, ")");
        let commands_before = collect_commands(&params_caption, EXEC_BEFORE_TAG, ")");
        let commands_after = collect_commands(&params_caption, EXEC_AFTER_TAG, ")");
        let color_vars = collect_colors(&params_caption);

        Ok(Template {
            self_path: path.to_string(),
            conf_path,
            conf_caption: config_caption,
            color_format,
            commands_before,
            commands_after,
            color_vars,
        })
    }

    pub fn apply(&self, hex_colors: Vec<String>) {
        self.exec_before();

        let mut config = self.conf_caption.clone();
        let mut color_values: Vec<ColorValue> = Vec::new();

        for color_var in &self.color_vars {
            if color_var.name.contains("{br}") {
                for i in 1..20 {
                    let mut lighter = ColorValue::from_hex(
                        &color_var.name.replace("{br}", &format!("LR{i}")),
                        &hex_colors[color_var.index as usize],
                    );
                    let mut darker = ColorValue::from_hex(
                        &color_var.name.replace("{br}", &format!("DR{i}")),
                        &hex_colors[color_var.index as usize],
                    );

                    if let Some(color) = &color_var.constant_value {
                        if let Color::HEX(hex) = color {
                            lighter.set_value_from_hex(&hex.value);
                            darker.set_value_from_hex(&hex.value);
                        } else if let Color::RGB(rgb) = color {
                            lighter.set_value_from_rgb(rgb.r, rgb.g, rgb.b);
                            darker.set_value_from_rgb(rgb.r, rgb.g, rgb.b);
                        }
                    }

                    if color_var.invert {
                        lighter.invert();
                        darker.invert();
                    }

                    lighter.add_brightness((i * 10) + color_var.brightness);
                    darker.add_brightness((i * -10) + color_var.brightness);

                    color_values.push(lighter);
                    color_values.push(darker);
                }
            }
            let mut color_value = ColorValue::from_hex(
                &color_var.name.replace("{br}", ""),
                &hex_colors[color_var.index as usize],
            );

            if let Some(color) = &color_var.constant_value {
                if let Color::HEX(hex) = color {
                    color_value.set_value_from_hex(&hex.value);
                } else if let Color::RGB(rgb) = color {
                    color_value.set_value_from_rgb(rgb.r, rgb.g, rgb.b);
                }
            }

            if color_var.invert {
                color_value.invert()
            }

            color_value.add_brightness(color_var.brightness);

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

        let _ = fs::write(expand_user(&self.conf_path), config);

        self.exec_after();
    }

    fn exec_before(&self) {
        for command in &self.commands_before {
            if !command.is_empty() {
                system(command);
            }
        }
    }

    fn exec_after(&self) {
        for command in &self.commands_after {
            if !command.is_empty() {
                spawn(command);
            }
        }
    }
}

impl FromStr for Template {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Template::new(&expand_user(s))
    }
}

#[derive(Clone)]
pub struct ColorVariable {
    name: String,
    index: usize,
    brightness: i32,
    invert: bool,
    constant_value: Option<Color>,
}

impl ColorVariable {
    pub fn new(
        name: String,
        index: usize,
        brightness: i32,
        invert: bool,
        constant_value: Option<Color>,
    ) -> Self {
        ColorVariable {
            name,
            index,
            brightness,
            invert,
            constant_value,
        }
    }
}

use std::fs;

use crate::{
    colorscheme::colors::{Color, HEX, RGB},
    expand_user,
};

use super::{
    tags::{COLOR_TAG, CONFIG_MARK, HEX_TAG, INCLUDE_TAG, RGB_TAG, TAGS},
    template::ColorVariable,
};

pub fn parse_template(raw_template: String) -> (Vec<String>, String) {
    let mut section_config = false;

    let mut params_caption: Vec<String> = Vec::new();
    let mut config_caption: Vec<String> = Vec::new();

    for line in raw_template.lines() {
        if line.trim() == CONFIG_MARK {
            section_config = true;
            continue;
        }

        if !section_config {
            if let Some(valid_line) = validate_line(line) {
                params_caption.push(valid_line);
            }
            continue;
        }

        config_caption.push(line.to_string());
    }

    params_caption = apply_include(params_caption);

    (params_caption, config_caption.join("\n"))
}

pub fn collect_commands(caption: &Vec<String>, prefix: &str, suffix: &str) -> Vec<String> {
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

pub fn collect_command(caption: &Vec<String>, prefix: &str, suffix: &str) -> String {
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

pub fn collect_colors(caption: &Vec<String>) -> Vec<ColorVariable> {
    let mut res = Vec::new();

    for command in collect_commands(caption, COLOR_TAG, ")") {
        let arguments: Vec<&str> = command.split(",").collect();

        if !arguments.len() < 2 {
            continue;
        }

        let name = arguments[0].trim().to_string();
        let index = arguments[1]
            .trim()
            .parse::<usize>()
            .unwrap_or(0)
            .clamp(0, 15);
        let mut brightness = 0;
        let mut invert = false;

        if arguments.len() > 2 {
            brightness = arguments[2].trim().parse().unwrap_or(0);
        }
        if arguments.len() > 3 {
            invert = matches!(arguments[3].trim(), "1" | "true" | "True");
        }

        res.push(ColorVariable::new(name, index, brightness, invert, None))
    }

    for command in collect_commands(caption, HEX_TAG, ")") {
        let arguments: Vec<&str> = command.split(",").collect();

        if arguments.len() != 2 {
            continue;
        }

        let name = arguments[0].trim().to_string();
        let index: usize = 0;
        let value = arguments[1].trim().to_string();

        if value.len() != 6 {
            continue;
        }

        let hex = Color::HEX(HEX { value });
        let brightness = 0;
        let invert = false;

        res.push(ColorVariable::new(
            name,
            index,
            brightness,
            invert,
            Some(hex),
        ))
    }

    for command in collect_commands(caption, RGB_TAG, ")") {
        let arguments: Vec<&str> = command.split(",").collect();

        if arguments.len() != 4 {
            continue;
        }

        let name = arguments[0].trim().to_string();
        let index: usize = 0;

        let r_value = arguments[1].trim().to_string().parse::<u8>();
        if r_value.is_err() {
            continue;
        }
        let r = r_value.unwrap();

        let g_value = arguments[2].trim().to_string().parse::<u8>();
        if g_value.is_err() {
            continue;
        }
        let g = g_value.unwrap();

        let b_value = arguments[3].trim().to_string().parse::<u8>();
        if b_value.is_err() {
            continue;
        }
        let b = b_value.unwrap();

        let brightness = 0;
        let invert = false;

        let rgb = Color::RGB(RGB { r, g, b });

        res.push(ColorVariable::new(
            name,
            index,
            brightness,
            invert,
            Some(rgb),
        ))
    }

    res
}

fn validate_line(s: &str) -> Option<String> {
    let s = remove_comment(s);
    let s = s.trim();

    for tag in TAGS {
        if s.starts_with(tag) && s.ends_with(")") {
            return Some(s.to_string());
        }
    }

    None
}

fn remove_comment(s: &str) -> String {
    if let Some(i) = s.find("//") {
        return s[0..i].to_string();
    }
    s.to_string()
}

fn apply_include(caption: Vec<String>) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();

    for line in caption {
        if !line.starts_with(INCLUDE_TAG) || !line.ends_with(")") {
            if let Some(valid_line) = validate_line(&line) {
                res.push(valid_line);
            }
            continue;
        }

        if let Ok(caption) =
            fs::read_to_string(expand_user(&line[INCLUDE_TAG.len()..line.len() - 1]))
        {
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

use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use crate::utils::spawn;

pub struct Template {
    pub temp_path: String,
    pub conf_path: String,
    pub use_quotes: bool,
    pub use_sharps: bool,
    pub opacity: String,
    pub command: String,
}

pub struct ColorVariable {
    pub name: String,
    pub value: usize,
    pub brightness: i32,
}

fn get_templates(data: Value) -> Vec<Template> {
    let mut res: Vec<Template> = Vec::new();
    for raw_template in data.as_array().unwrap() {
        res.push(Template {
            temp_path: String::from(raw_template["template_path"].as_str().unwrap()),
            conf_path: String::from(raw_template["config_path"].as_str().unwrap()),
            use_quotes: raw_template["use_quotes"].as_bool().unwrap(),
            use_sharps: raw_template["use_sharps"].as_bool().unwrap(),
            opacity: String::from(raw_template["opacity"].as_str().unwrap()),
            command: String::from(raw_template["command"].as_str().unwrap()),
        })
    }
    return res;
}
fn get_color_variables(data: Value) -> Vec<ColorVariable> {
    let mut colors: Vec<ColorVariable> = Vec::new();
    for raw_variable in data.as_array().unwrap() {
        colors.push(ColorVariable {
            name: String::from(raw_variable["name"].as_str().unwrap()),
            value: raw_variable["value"].as_u64().unwrap() as usize,
            brightness: raw_variable["brightness"].as_i64().unwrap() as i32,
        })
    }
    return colors;
}

fn process_color(color: u8, brightness: i32) -> String {
    if color as i32 + brightness >= 255 {
        return String::from("FF");
    }
    if color as i32 + brightness <= 0 {
        return String::from("00");
    }

    let tmp: i32 = color as i32 + &brightness;
    let hex = format!("{:X}", tmp);

    if hex.len() == 1 {
        return format!("0{}", hex);
    }
    return hex;
}

fn get_wal_colors(path: String) -> Vec<String> {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let tmp: Vec<&str> = data.split("\n").collect();
    let mut res: Vec<String> = Vec::new();

    for color in tmp {
        res.push(String::from(color));
    }

    return res;
}

pub fn apply_templates(
    templates_value: Value,
    variables_value: Value,
    wal_color_path: String,
) {
    let templates: Vec<Template> = get_templates(templates_value);
    let variables: Vec<ColorVariable> = get_color_variables(variables_value);
    let colors = get_wal_colors(wal_color_path);

    for template in templates {
        let mut file = File::open(template.temp_path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        for variable in &variables {
            let value = &colors[variable.value][1..];
            let br = variable.brightness;
            let r: u8 = u8::from_str_radix(&value[0..2], 16).unwrap();
            let g: u8 = u8::from_str_radix(&value[2..4], 16).unwrap();
            let b: u8 = u8::from_str_radix(&value[4..6], 16).unwrap();

            let mut color = format!(
                "#{}{}{}{}",
                process_color(r, br),
                process_color(g, br),
                process_color(b, br),
                template.opacity
            );

            if !template.use_sharps {
                color = String::from(&color[1..]);
            }
            if template.use_quotes {
                color = format!("{}{}{}", '"', color, '"');
            }

            data = data.replace(&variable.name, &color);
        }
        let mut file = File::create(template.conf_path).expect("Failed to create file");
        file.write_all(data.as_bytes())
            .expect("Failed to write to file");
        spawn(template.command);
    }
}

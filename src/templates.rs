use crate::utils::{spawn, system};
use std::clone::Clone;
use std::fs::File;
use std::io::{self, Read, Write};
use std::thread;

#[derive(Clone)]
pub struct Template {
    pub temp_path: String,
    pub conf_path: String,
    pub use_quotes: bool,
    pub use_sharps: bool,
    pub opacity: String,
    pub pre_command: String,
    pub post_command: String,
}

impl Template {
    pub fn new(
        temp_path: String,
        conf_path: String,
        use_quotes: bool,
        use_sharps: bool,
        opacity: String,
        command: String,
    ) -> Self {
        let mut pre_command: String = String::from("");
        let mut post_command: String = String::from("");

        let parsed_command: Vec<&str> = command.split("||").collect();

        match parsed_command.len() {
            1 => {
                post_command = String::from(parsed_command[0]);
            }
            2 => {
                pre_command = String::from(parsed_command[0]);
                post_command = String::from(parsed_command[1]);
            }
            _ => {}
        }

        Template {
            temp_path,
            conf_path,
            use_quotes,
            use_sharps,
            opacity,
            pre_command,
            post_command,
        }
    }
}

#[derive(Clone)]
pub struct ColorVariable {
    pub name: String,
    pub value: usize,
    pub brightness: i32,
    pub inverted: bool,
    pub is_constant: bool,
    pub constant_value: String,
}

impl ColorVariable {
    pub fn new(name: String, value: usize, brightness: i32, inverted: bool, is_constant: bool, constant_value: String) -> Self {
        ColorVariable {
            name,
            value,
            brightness,
            inverted,
            is_constant,
            constant_value,
        }
    }

    fn process_color(&self, color: u8) -> String {
        let mut _res = String::new();
        if color as i32 + self.brightness >= 255 {
            _res = String::from("FF");
        } else if color as i32 + self.brightness <= 0 {
            _res = String::from("00");
        } else {
            let mut tmp: i32 = color as i32 + self.brightness;
            if self.inverted {
                tmp = 255 - tmp
            }
            let hex = format!("{:X}", tmp);

            if hex.len() == 1 {
                _res = format!("0{}", hex);
            } else {
                _res = format!("{}", hex);
            }
        }

        _res
    }

    pub fn process_colors(&self, value: &str) -> String {
        let r: u8 = u8::from_str_radix(&value[0..2], 16).unwrap();
        let g: u8 = u8::from_str_radix(&value[2..4], 16).unwrap();
        let b: u8 = u8::from_str_radix(&value[4..6], 16).unwrap();

        let hex_r = self.process_color(r);
        let hex_g = self.process_color(g);
        let hex_b = self.process_color(b);

        return format!("{}{}{}", hex_r, hex_g, hex_b);
    }
}

fn get_colors_from_scheme(path: String) -> Vec<String> {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let tmp: Vec<&str> = data.split("\n").collect();
    let mut res: Vec<String> = Vec::new();

    for color in tmp {
        res.push(String::from(color));
    }

    res
}

fn apply_template(
    template: Template,
    variables: Vec<ColorVariable>,
    colors: Vec<String>,
) -> Result<(), io::Error> {
    system(&template.pre_command);
    let mut file = File::open(template.temp_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    for variable in &variables {
        let value = &colors[variable.value][1..];

        let mut color: String;

        if variable.is_constant {
            color = format!("#{}{}", variable.process_colors(&variable.constant_value), template.opacity);
        } else {
            color = format!("#{}{}", variable.process_colors(&value), template.opacity);
        }

        if !template.use_sharps {
            color = String::from(&color[1..]);
        }
        if template.use_quotes {
            color = format!("{}{}{}", '"', color, '"');
        }

        data = data.replace(&variable.name, &color);
    }
    let mut file = File::create(template.conf_path)?;
    file.write_all(data.as_bytes())?;
    spawn(&template.post_command);

    Ok(())
}

pub fn apply_templates(
    templates: Vec<Template>,
    variables: Vec<ColorVariable>,
    color_scheme_path: String,
) {
    let colors = get_colors_from_scheme(color_scheme_path);

    let mut threads = Vec::new();

    for template in templates {
        let _template = template.clone();
        let local_colors = colors.to_vec();
        let local_variables = variables.to_vec();
        let thread =
            thread::spawn(move || apply_template(_template, local_variables, local_colors));
        threads.push(thread);
    }

    for thread in threads {
        match thread.join().unwrap() {
            Ok(()) => continue,
            Err(err) => {
                println!("{}", err)
            }
        }
    }
}

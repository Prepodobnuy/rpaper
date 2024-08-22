use std::thread;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::clone::Clone;
use crate::utils::spawn;

pub struct Template {
    pub temp_path: String,
    pub conf_path: String,
    pub use_quotes: bool,
    pub use_sharps: bool,
    pub opacity: String,
    pub command: String,
}

#[derive(Clone)]
pub struct ColorVariable {
    pub name: String,
    pub value: usize,
    pub brightness: i32,
    pub inverted: bool,
}

impl ColorVariable {
    fn process_color(&self, color: u8) -> String {
        let mut res = String::new();
        if color as i32 + self.brightness >= 255 {
            res = String::from("FF");
        }
        else if color as i32 + self.brightness <= 0 {
            res = String::from("00");
        }
        else {
            let mut tmp: i32 = color as i32 + self.brightness;
            if self.inverted {tmp = 255 - tmp}
            let hex = format!("{:X}", tmp);  

            if hex.len() == 1 {
                res = format!("0{}", hex);
            } 
            else {
                res = format!("{}", hex);
            }

        }

        return res;
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
    let mut variables: Vec<ColorVariable> = Vec::new();
    for raw_variable in data.as_array().unwrap() {
        let mut name = String::from(raw_variable["name"].as_str().unwrap());
        let value = raw_variable["value"].as_u64().unwrap_or(0) as usize;
        let brightness = raw_variable["brightness"].as_i64().unwrap_or(0) as i32;
        let inverted = raw_variable["inverted"].as_bool().unwrap_or(false);
        
        if name.contains("{br}") {
            let oldname = name;
            name = oldname.replace("{br}", "");
            for i in 1..11 {
                variables.push(ColorVariable {
                    name: oldname.replace("{br}", &format!("d{}", i)),
                    value,
                    brightness: brightness - i*10,
                    inverted,
                });
            }
            for i in 1..11 {
                variables.push(ColorVariable {
                    name: oldname.replace("{br}", &format!("l{}", i)),
                    value,
                    brightness: brightness + i*10,
                    inverted,
                });
            }
        }

        variables.push(ColorVariable {
            name,
            value, 
            brightness,
            inverted,
        })
    }
    return variables;
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
    let colors = get_wal_colors(wal_color_path);

    let templates: Vec<Template> = get_templates(templates_value);
    let variables: Vec<ColorVariable> = get_color_variables(variables_value);

    for template in templates {
        let local_colors = colors.to_vec();
        let local_variables = variables.to_vec();
        thread::spawn(move || {
            let mut file = File::open(template.temp_path).unwrap();
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
    
    
            for variable in &local_variables {
                let value = &local_colors[variable.value][1..];
    
                let mut color = format!(
                    "#{}{}",
                    variable.process_colors(&value),
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
        });
    }
}

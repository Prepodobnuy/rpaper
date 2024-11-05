use std::clone::Clone;
use std::fs::{self, File};
use std::io::Write;
use std::thread;

// comment

use crate::utils::{parse_path, spawn, system, read_data};
use crate::log::warn;

#[derive(Clone)]
pub struct Template {
    pub path: String,
    pub format: String,
    pub before: String,
    pub after: String,
    pub caption: String,
}

impl Template {
    pub fn read(template_path: &str) -> Result<Template, String> {
        let strings: Vec<&str>;

        if let Ok(data) = fs::read_to_string(template_path) {
            strings = data.split("\n").collect();
            let mut reading_params = false;
            let mut reading_caption = false;

            let mut path: String = String::from("");
            let mut format: String = String::from("");
            let mut before: String = String::from("");
            let mut after: String = String::from("");
            let mut caption: String = String::from("");

            for string in strings {
                if !reading_caption && string.trim() == "[param]" {
                    reading_params = true;
                    continue;
                }
                if string.trim() == "[caption]" {
                    reading_params = false;
                    reading_caption = true;
                    continue;
                }
                if reading_params {
                    if string == "" {continue;}
                    if let Some(char) = string.chars().next() {
                        if char == '#' {continue;}
                    }
                    if !string.contains(":") {continue;}
                    
                    let splitted: Vec<&str> = string.split(":").collect();
                    if splitted[1].is_empty() || splitted[0].is_empty() {continue;}
                    match splitted[0] {
                        "path" => path = parse_path(splitted[1]),
                        "format" => format = splitted[1].to_string(),
                        "before" => before = splitted[1].to_string(),
                        "after" => after = splitted[1].to_string(),
                        _ => {},
                    }
                }
                if reading_caption {
                    caption += string;
                    caption += "\n";
                }
            }
            return Ok(Template {
                path,
                format,
                before,
                after,
                caption,
            });
        }

        Err(format!("Error reading file: {}", template_path))
    }

    pub fn apply(&self, color_variables: &Vec<ColorVariable>) -> Result<(), String> {
        // run before command
        system(&self.before);

        let mut result: String = self.caption.clone();

        // filling template with colors
        for variable in color_variables {
            if variable.name.contains("{br}") {
                for i in 0..255 {
                    let mut tmp_color = variable.value.clone();
                    tmp_color.clamp(i);
                    let color = self.format
                        .replace("{HEX}", &tmp_color.to_hex())
                        .replace("{R}", &tmp_color.r.to_string())
                        .replace("{G}", &tmp_color.g.to_string())
                        .replace("{B}", &tmp_color.b.to_string());
                    
                    let name = variable.name.replace("{br}", &format!("BR{}", i));
                    
                    result = result.replace(&name,&color);
            }
            let color = self.format
                .replace("{HEX}", &variable.value.to_hex())
                .replace("{R}", &variable.value.r.to_string())
                .replace("{G}", &variable.value.g.to_string())
                .replace("{B}", &variable.value.b.to_string());

            let name = variable.name.clone().replace("{br}", "");
                    
            result = result.replace(&name,&color);
            }
        }

        let file = File::create(self.path.clone());
        match file {
            Ok(mut file) => {
                match file.write_all(result.as_bytes()) {
                    Ok(_) => {},
                    Err(_) => {
                        warn(&format!("Error writing to {}", self.path));
                        warn("Template config will not be updated.");
                    },
                };

            },
            Err(_) => {},
        }

        // run after command
        spawn(&self.after);

        Ok(())
    }
}


#[derive(Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Color {
            r,
            g,
            b,
        }
    }

    fn clamp(&mut self, brightness: u8) {
        let m: f32 = (self.r + self.g + self.b) as f32 / 3.0;
        let d: f32 = m / brightness as f32;

        let mut r: f32 = self.r as f32 / d;
        let mut g: f32 = self.g as f32 / d;
        let mut b: f32 = self.b as f32 / d;

        r = if r > 255.0 {255.0} else {r};
        g = if g > 255.0 {255.0} else {g};
        b = if b > 255.0 {255.0} else {b};

        r = if r < 0.0 {0.0} else {r};
        g = if g < 0.0 {0.0} else {g};
        b = if b < 0.0 {0.0} else {b};

        self.r = r as u8;
        self.g = g as u8;
        self.b = b as u8;
    }

    fn to_hex(& self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[derive(Clone)]
pub struct ColorVariable {
    name: String,
    value: Color,
}

impl ColorVariable {
    pub fn new(name: &str, colorscheme: &Vec<String>, index: usize, inverted: bool, brightness: i32) -> Self {
        // name:         name of color variable
        // colorscheme : vector of colors from pallete
        // index:        index of color in colorscheme
        // inverted:     boolean responsible for inverting color
        let value = &colorscheme[index];

        let mut r: u8 = match u8::from_str_radix(&value[0..2], 16) {
            Ok(value) => value,
            Err(_) => {
                warn(&format!("Error in processing red in ColorVariable {}", name));
                warn(&format!("Value of red color is setted to {}", 127));
                127
            },
        };
        let mut g: u8 = match u8::from_str_radix(&value[2..4], 16) {
            Ok(value) => value,
            Err(_) => {
                warn(&format!("Error in processing green in ColorVariable {}", name));
                warn(&format!("Value of green color is setted to {}", 127));
                127
            },
        };
        let mut b: u8 = match u8::from_str_radix(&value[4..6], 16) {
            Ok(value) => value,
            Err(_) => {
                warn(&format!("Error in processing blue in ColorVariable {}", name));
                warn(&format!("Value of blue color is setted to {}", 127));
                127
            },
        };

        let mut temp_r = r as i32 + brightness;
        let mut temp_g = g as i32 + brightness;
        let mut temp_b = b as i32 + brightness;

        temp_r = if temp_r > 255 {255} else {temp_r};
        temp_g = if temp_g > 255 {255} else {temp_g};
        temp_b = if temp_b > 255 {255} else {temp_b};

        temp_r = if temp_r < 0 {0} else {temp_r};
        temp_g = if temp_g < 0 {0} else {temp_g};
        temp_b = if temp_b < 0 {0} else {temp_b};

        r = temp_r as u8;
        g = temp_g as u8;
        b = temp_b as u8;

        if inverted {
            r = 255 - r;
            g = 255 - g;
            b = 255 - b;
        }
        
        let color: Color = Color::new(r, g, b);

        ColorVariable {
            name: name.to_string(),
            value: color,
        }
    }
}

pub fn apply_templates(
    templates: Vec<Template>,
    variables: Vec<ColorVariable>,
) {
    let mut threads = Vec::new();

    for template in templates {
        let template = template.clone();
        let color_variables = variables.to_vec();
        let thread =
            thread::spawn(move || template.apply(&color_variables));
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

pub fn fill_color_variables(vars_path: &str, color_scheme_file_path: &str) -> Vec<ColorVariable> {
    let colors = fs::read_to_string(color_scheme_file_path).unwrap_or("".to_string());
    let colorscheme: Vec<String> = colors
        .lines()
        .map(|color| {
            if color.len() > 1 {
                return color[1..].to_string();
            } 
            String::from("")
        })
        .collect();

    let mut color_vars: Vec<ColorVariable> = Vec::new();

    let color_vars_data = read_data(vars_path);
    let raw_color_vars = color_vars_data.as_array().unwrap();

    for raw_color_var in raw_color_vars {
        let name = raw_color_var["name"].as_str().unwrap_or("2tliuervhgp9834tygp3k4jhvdsfgvt239tiusc@!$!@*&TKAHCGO*!&#RTFJHH");
        let index = raw_color_var["value"].as_u64().unwrap_or(0) as usize;
        let brightness = raw_color_var["brightness"].as_i64().unwrap_or(0) as i32;
        let inverted = raw_color_var["inverted"].as_bool().unwrap_or(false);
        if name.contains("{br}") {
            for i in 1..10 {
                color_vars.push(ColorVariable::new(
                    &name.replace("{br}", &format!("DR{}{{br}}", i)),
                    &colorscheme,
                    index,
                    inverted,
                    brightness - (i * 10)
                ));
                color_vars.push(ColorVariable::new(
                    &name.replace("{br}", &format!("LR{}{{br}}", i)),
                    &colorscheme,
                    index,
                    inverted,
                    brightness + (i * 10)
                ));
            }
        }
        color_vars.push(ColorVariable::new(name, &colorscheme, index, inverted, brightness));
    }

    color_vars
}
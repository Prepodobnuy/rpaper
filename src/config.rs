use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::Read;

use crate::displays::Display;
use crate::templates::{ColorVariable, Template};
use crate::utils::parse_path;

pub struct ArgvParser {
    argv: Vec<String>,
}

impl ArgvParser {
    pub fn new() -> Self {
        let argv: Vec<String> = env::args().collect();
        ArgvParser { argv }
    }
}
impl ArgvParser {
    pub fn get_config_path(&self, default_config_path: String) -> String {
        let mut res = default_config_path;

        if self.argv.len() <= 2 {
            return res;
        }

        for (i, param) in self.argv.iter().enumerate() {
            match param.as_str() {
                "-c" | "--conf" => {
                    res = parse_path(&self.argv[i + 1]);
                    break;
                }
                _ => {}
            }
        }
        res
    }
    fn wallpaper_cache_only(&self) -> bool {
        let mut res = false;
        if self.argv.len() <= 2 {
            return res;
        }

        for param in self.argv.iter() {
            match param.as_str() {
                "--cache-wallpaper" => {
                    res = true;
                    break;
                }
                _ => {}
            }
        }

        res
    }
    fn color_scheme_cache_only(&self) -> bool {
        let mut res = false;
        if self.argv.len() <= 2 {
            return res;
        }

        for param in self.argv.iter() {
            match param.as_str() {
                "--cache-color" => {
                    res = true;
                    break;
                }
                _ => {}
            }
        }

        res
    }
    fn cache_only(&self) -> bool {
        let mut res = false;
        if self.argv.len() <= 2 {
            return res;
        }

        for param in self.argv.iter() {
            match param.as_str() {
                "--cache" => {
                    res = true;
                    break;
                }
                _ => {}
            }
        }

        res
    }
}

fn read_data(data_path: &str) -> Value {
    let mut file = File::open(data_path).unwrap();
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).unwrap();

    let data: Value = serde_json::from_str(&json_data).unwrap();

    return data;
}

#[derive(Clone)]
pub struct Config {
    pub cached_images_path: String,
    pub color_scheme_file: String,

    pub set_wallpaper_command: String,
    pub wallpaper_resize_backend: String,

    pub cache_colorscheme: bool,
    pub apply_templates: bool,
    pub cache_wallpaper: bool,
    pub set_wallpaper: bool,

    pub rwal_cache_dir: String,
    pub rwal_thumb: (u32, u32),
    pub rwal_accent_color: u32,
    pub rwal_clamp_min_v: f32,
    pub rwal_clamp_max_v: f32,

    pub image_operations: ImageOperations,

    pub displays: Vec<Display>,

    pub templates: Vec<Template>,
    pub variables: Vec<ColorVariable>,
}

#[derive(Clone)]
pub struct ImageOperations {
    pub change_contrast: bool,
    pub change_brightness: bool,
    pub change_huerotate: bool,
    pub change_blur: bool,
    pub image_flip_h: bool,
    pub image_flip_v: bool,
    pub invert_image: bool,
    pub contrast: f32,
    pub brightness: i32,
    pub huerotate: i32,
    pub blur: f32,
}

impl ImageOperations {
    pub fn new(
        change_contrast: bool,
        change_brightness: bool,
        change_huerotate: bool,
        change_blur: bool,
        image_flip_h: bool,
        image_flip_v: bool,
        invert_image: bool,
        contrast: f32,
        brightness: i32,
        huerotate: i32,
        blur: f32,
    ) -> Self {
        ImageOperations {
            change_contrast,
            change_brightness,
            change_huerotate,
            change_blur,
            image_flip_h,
            image_flip_v,
            invert_image,
            contrast,
            brightness,
            huerotate,
            blur,
        }
    }
}

impl Config {
    pub fn new(config_path: &str, argv_parser: ArgvParser) -> Self {
        // json raw data
        let config_data = read_data(config_path);
        let templates_data =
            read_data(&parse_path(config_data["templates_path"].as_str().unwrap()));
        let colorvars_data =
            read_data(&parse_path(config_data["variables_path"].as_str().unwrap()));

        // path
        let cached_images_path = parse_path(
            config_data["cached_wallpapers_dir"]
                .as_str()
                .unwrap_or("~/.config/rpaper/templates.json"),
        );
        let color_scheme_file = parse_path(
            config_data["color_scheme_file"]
                .as_str()
                .unwrap_or("~/.config/rpaper/color_variables.json"),
        );
        // command
        let set_wallpaper_command =
            String::from(config_data["set_wallpaper_command"].as_str().unwrap());

        let wallpaper_resize_backend =
            String::from(config_data["wallpaper_resize_backend"].as_str().unwrap());
        //booleans
        let mut cache_colorscheme = config_data["cache_colorscheme"].as_bool().unwrap_or(true);
        let mut apply_templates = config_data["apply_templates"].as_bool().unwrap_or(true);
        let mut cache_wallpaper = config_data["cache_wallpaper"].as_bool().unwrap_or(true);
        let mut set_wallpaper = config_data["set_wallpaper"].as_bool().unwrap_or(true);
        if argv_parser.cache_only() {
            apply_templates = false;
            set_wallpaper = false;
        }
        if argv_parser.color_scheme_cache_only() {
            apply_templates = false;
            set_wallpaper = false;
            cache_wallpaper = false;
        }
        if argv_parser.wallpaper_cache_only() {
            apply_templates = false;
            set_wallpaper = false;
            cache_colorscheme = false;
        }

        //rwal
        let rwal_cache_dir = parse_path(
            config_data["rwal_cache_dir"]
                .as_str()
                .unwrap_or("~/.cache/rpaper/rwal"),
        );
        let rwal_thumb_w = config_data["rwal_thumb_w"].as_u64().unwrap_or(200) as u32;
        let rwal_thumb_h = config_data["rwal_thumb_h"].as_u64().unwrap_or(200) as u32;
        let rwal_thumb = (rwal_thumb_w, rwal_thumb_h);
        let rwal_accent_color = config_data["rwal_accent_color"].as_u64().unwrap_or(5) as u32;
        let rwal_clamp_min_v = config_data["rwal_clamp_min_v"].as_f64().unwrap_or(170.0) as f32;
        let rwal_clamp_max_v = config_data["rwal_clamp_max_v"].as_f64().unwrap_or(200.0) as f32;
        // ImageOperations
        let image_operations = get_image_operations(&config_data);
        // displays
        let displays = get_displays(&config_data);
        // templates
        let templates = get_templates(templates_data);
        // variables
        let variables = get_variables(colorvars_data);

        Config {
            //path
            cached_images_path,
            color_scheme_file,
            //command
            set_wallpaper_command,
            wallpaper_resize_backend,
            //booleans
            cache_colorscheme,
            apply_templates,
            cache_wallpaper,
            set_wallpaper,
            //rwal
            rwal_cache_dir,
            rwal_thumb,
            rwal_accent_color,
            rwal_clamp_min_v,
            rwal_clamp_max_v,
            //ImageOperations
            image_operations,
            //displays
            displays,
            //templates
            templates,
            //variable
            variables,
        }
    }
}

fn get_image_operations(config_data: &Value) -> ImageOperations {
    let change_contrast = config_data["change_contrast"].as_bool().unwrap_or(false);
    let change_brightness = config_data["change_brightness"].as_bool().unwrap_or(false);
    let change_huerotate = config_data["change_huerotate"].as_bool().unwrap_or(false);
    let change_blur = config_data["change_blur"].as_bool().unwrap_or(false);
    let image_flip_h = config_data["image_flip_h"].as_bool().unwrap_or(false);
    let image_flip_v = config_data["image_flip_v"].as_bool().unwrap_or(false);
    let invert_image = config_data["invert_image"].as_bool().unwrap_or(false);
    let contrast = config_data["contrast"].as_f64().unwrap_or(0.0) as f32;
    let brightness = config_data["brightness"].as_i64().unwrap_or(0) as i32;
    let huerotate = config_data["huerotate"].as_i64().unwrap_or(0) as i32;
    let blur = config_data["blur"].as_f64().unwrap_or(0.0) as f32;
    let image_operations = ImageOperations::new(
        change_contrast,
        change_brightness,
        change_huerotate,
        change_blur,
        image_flip_h,
        image_flip_v,
        invert_image,
        contrast,
        brightness,
        huerotate,
        blur,
    );
    image_operations
}

fn get_displays(config_data: &Value) -> Vec<Display> {
    let mut displays: Vec<Display> = Vec::new();
    for raw_display in config_data["displays"].as_array().unwrap() {
        let w = raw_display["width"].as_u64().unwrap() as u32;
        let h = raw_display["height"].as_u64().unwrap() as u32;
        let x = raw_display["margin-left"].as_u64().unwrap() as u32;
        let y = raw_display["margin-top"].as_u64().unwrap() as u32;
        let name = String::from(raw_display["name"].as_str().unwrap());

        displays.push(Display::new(w, h, x, y, name))
    }
    displays
}

fn get_templates(templates_data: Value) -> Vec<Template> {
    let mut templates: Vec<Template> = Vec::new();
    for raw_template in templates_data.as_array().unwrap() {
        let temp_path = String::from(raw_template["template_path"].as_str().unwrap());
        let conf_path = String::from(raw_template["config_path"].as_str().unwrap());
        let use_quotes = raw_template["use_quotes"].as_bool().unwrap();
        let use_sharps = raw_template["use_sharps"].as_bool().unwrap();
        let opacity = String::from(raw_template["opacity"].as_str().unwrap());
        let command = String::from(raw_template["command"].as_str().unwrap());

        templates.push(Template::new(
            temp_path, conf_path, use_quotes, use_sharps, opacity, command,
        ));
    }
    templates
}

fn get_variables(colorvars_data: Value) -> Vec<ColorVariable> {
    let mut variables: Vec<ColorVariable> = Vec::new();
    for raw_variable in colorvars_data.as_array().unwrap() {
        let mut name = String::from(raw_variable["name"].as_str().unwrap());
        let value = raw_variable["value"].as_u64().unwrap_or(0) as usize;
        let brightness = raw_variable["brightness"].as_i64().unwrap_or(0) as i32;
        let inverted = raw_variable["inverted"].as_bool().unwrap_or(false);
        if name.contains("{br}") {
            let oldname = name;
            name = oldname.replace("{br}", "");
            for i in 1..11 {
                variables.push(ColorVariable::new(
                    oldname.replace("{br}", &format!("d{}", i)),
                    value,
                    brightness - (i * 10),
                    inverted,
                ));
            }
            for i in 1..11 {
                variables.push(ColorVariable::new(
                    oldname.replace("{br}", &format!("l{}", i)),
                    value,
                    brightness + (i * 10),
                    inverted,
                ));
            }
        }
        variables.push(ColorVariable::new(name, value, brightness, inverted));
    }
    variables
}

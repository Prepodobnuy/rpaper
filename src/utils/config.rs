use std::fs::File;
use std::io::Read;
use serde_json::Value;

use crate::displays::Display;
use crate::templates::Template;
use crate::utils::directory::expand_user;
use crate::argparser::Args;

use super::directory::{cache_dir, rwal_cache_dir};

#[derive(Clone)]
pub struct Config {
    pub cache_dir: String,
    pub scheme_file: String,

    pub wall_command: String,
    pub resize_algorithm: String,

    pub cache_scheme: bool,
    pub cache_walls: bool,
    pub set_templates: bool,
    pub set_walls: bool,

    pub image_operations: ImageOperations,
    pub rwal_params: RwalParams,

    pub displays: Vec<Display>,
    pub templates: Vec<Template>,
    pub vars_path: String,
}

#[derive(Clone)]
pub struct RwalParams {
    pub cache_dir: String,
    pub thumb_w: u32,
    pub thumb_h: u32,
    pub accent: u32,
    pub clamp_min: f32,
    pub clamp_max: f32,
}

#[derive(Clone)]
pub struct ImageOperations {
    pub flip_h: bool,
    pub flip_v: bool,
    pub invert: bool,
    pub contrast: f32,
    pub brightness: i32,
    pub huerotate: i32,
    pub blur: f32,
}

impl Config {
    pub fn new(config_path: &str, cache_only: bool) -> Self {
        let args = Args::new();

        // TODO simplify this mess
        let config_data = read_data(config_path);
        let template_paths: &Vec<Value> = config_data["templates"].as_array().unwrap();
        let vars_path = expand_user(match &args.rpaper_vars_path {
            Some(path) => path,
            None => config_data["vars_path"].as_str().unwrap(),
        });

        let cache_dir = cache_dir();

        let scheme_file = expand_user(match &args.rpaper_scheme_file {
            Some(path) => path,
            None => config_data["scheme_file"]
                .as_str()
                .unwrap_or("~/.config/rpaper/color_variables.json"),
        });

        let wall_command = (match &args.rpaper_wall_command {
            Some(result) => result,
            None => config_data["wall_command"].as_str().unwrap(),
        })
        .to_string();
        let resize_algorithm = (match &args.rpaper_resize_algorithm {
            Some(result) => result,
            None => config_data["resize_algorithm"]
                .as_str()
                .unwrap_or("Lanczos3"),
        })
        .to_string();

        let cache_scheme = match args.rpaper_cache_scheme {
            Some(result) => result,
            None => config_data["cache_scheme"].as_bool().unwrap_or(true),
        };
        let cache_walls = match args.rpaper_cache_walls {
            Some(result) => result,
            None => config_data["cache_walls"].as_bool().unwrap_or(true),
        };
        let mut set_templates = match args.rpaper_set_templates {
            Some(result) => result,
            None => config_data["set_templates"].as_bool().unwrap_or(true),
        };
        let mut set_walls = match args.rpaper_set_walls {
            Some(result) => result,
            None => config_data["set_walls"].as_bool().unwrap_or(true),
        };
        if cache_only {
            set_templates = false;
            set_walls = false;
        }

        let rwal_params = get_rwal_params(&config_data, &args);
        let image_operations = get_image_operations(&config_data, &args);

        let displays = get_displays(&config_data, args.displays);
        let templates = get_templates(template_paths, args.templates);

        Config {
            cache_dir,
            scheme_file,

            wall_command,
            resize_algorithm,

            cache_scheme,
            cache_walls,
            set_templates,
            set_walls,

            image_operations,
            rwal_params,

            displays,
            templates,
            vars_path,
        }
    }
}

fn get_image_operations(config_data: &Value, args: &Args) -> ImageOperations {
    let flip_h = match args.image_processing_h_flip {
        Some(val) => val,
        None => config_data["imgp_flip_h"].as_bool().unwrap_or(false),
    };
    let flip_v = match args.image_processing_v_flip {
        Some(val) => val,
        None => config_data["imgp_flip_v"].as_bool().unwrap_or(false),
    };
    let invert = match args.image_processing_invert {
        Some(val) => val,
        None => config_data["imgp_invert"].as_bool().unwrap_or(false),
    };

    let contrast = match args.image_processing_contrast {
        Some(val) => val,
        None => config_data["imgp_contrast"].as_f64().unwrap_or(0.0) as f32,
    };
    let brightness = match args.image_processing_brigtness {
        Some(val) => val,
        None => config_data["imgp_brightness"].as_i64().unwrap_or(0) as i32,
    };
    let huerotate = match args.image_processing_hue {
        Some(val) => val,
        None => config_data["imgp_huerotate"].as_i64().unwrap_or(0) as i32,
    };
    let blur = match args.image_processing_blur {
        Some(val) => val,
        None => config_data["imgp_blur"].as_f64().unwrap_or(0.0) as f32,
    };

    ImageOperations {
        flip_h,
        flip_v,
        invert,

        contrast,
        brightness,
        huerotate,
        blur,
    }
}

fn get_rwal_params(config_data: &Value, args: &Args) -> RwalParams {
    let cache_dir = rwal_cache_dir();
    let thumb_w = match args.rwal_thumb_w {
        Some(val) => val,
        None => config_data["rwal_thumb_w"].as_u64().unwrap_or(200) as u32,
    };
    let thumb_h = match args.rwal_thumb_h {
        Some(val) => val,
        None => config_data["rwal_thumb_h"].as_u64().unwrap_or(200) as u32,
    };
    let accent = match args.rwal_accent {
        Some(val) => val,
        None => config_data["rwal_accent_color"].as_u64().unwrap_or(5) as u32,
    };
    let clamp_min = match args.rwal_clamp_min {
        Some(val) => val,
        None => config_data["rwal_clamp_min"].as_f64().unwrap_or(170.0) as f32,
    };
    let clamp_max = match args.rwal_clamp_max {
        Some(val) => val,
        None => config_data["rwal_clamp_max"].as_f64().unwrap_or(170.0) as f32,
    };
    RwalParams {
        cache_dir,
        thumb_w,
        thumb_h,
        accent,
        clamp_min,
        clamp_max,
    }
}

fn get_displays(config_data: &Value, raw_displays: Option<String>) -> Vec<Display> {
    let mut displays: Vec<Display> = Vec::new();

    if let Some(display_data) = raw_displays {
        for raw_display in display_data.split(",") {
            let display_params: Vec<&str> = raw_display.split(":").collect();

            let name: String = display_params[0].to_string();
            let w: u32 = display_params[1].parse().unwrap_or_else(|_| {
                panic!("Invalid {} width\nPerhaps you forgot to specify a value. Remember that the value must be an integer.", name)
            });
            let h: u32 = display_params[2].parse().unwrap_or_else(|_| {
                panic!("Invalid {} height\nPerhaps you forgot to specify a value. Remember that the value must be an integer.", name)
            });
            let x: u32 = display_params[3].parse().unwrap_or_else(|_| {
                panic!("Invalid {} x position\nPerhaps you forgot to specify a value. Remember that the value must be an integer.", name)
            });
            let y: u32 = display_params[4].parse().unwrap_or_else(|_| {
                panic!("Invalid {} y position\nPerhaps you forgot to specify a value. Remember that the value must be an integer.", name)
            });

            displays.push(Display::new(w, h, x, y, name))
        }
        return displays;
    }

    for raw_display in config_data["displays"].as_array().unwrap() {
        let name = String::from(raw_display["name"].as_str().unwrap());
        let w = raw_display["w"].as_u64().unwrap() as u32;
        let h = raw_display["h"].as_u64().unwrap() as u32;
        let x = raw_display["x"].as_u64().unwrap() as u32;
        let y = raw_display["y"].as_u64().unwrap() as u32;

        displays.push(Display::new(w, h, x, y, name))
    }
    displays
}

fn get_templates(template_paths: &Vec<Value>, _templates: Option<String>) -> Vec<Template> {
    let mut templates: Vec<Template> = Vec::new();
    
    if let Some(_templates) = _templates {
        _templates.split(",")
            .for_each(|template_path| {
                if let Ok(template) = Template::read(&expand_user(template_path)) {
                    templates.push(template);
                }
            });
        return templates;
    }
    
    for template_path in template_paths {
        if let Some(path) = template_path.as_str() {
            if let Ok(template) = Template::read(&expand_user(path)) {
                templates.push(template);
            }
        }
    }
    templates
}

pub fn read_data(data_path: &str) -> Value {
    let mut file = File::open(data_path).unwrap();
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).unwrap();

    let data: Value = serde_json::from_str(&json_data).unwrap();

    data
}
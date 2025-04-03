pub mod display;
pub use display::Display;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub image: Option<String>,
    pub cache: bool,
    pub w_set: bool,
    pub w_cache: bool,
    pub c_set: bool,
    pub c_cache: bool,
    pub displays: Option<Vec<Display>>,
    pub templates: Option<Vec<String>>,
    pub resize_alg: Option<String>,
    pub set_command: Option<String>,
    pub contrast: Option<i32>,
    pub brightness: Option<f32>,
    pub hue: Option<i32>,
    pub blur: Option<f32>,
    pub invert: bool,
    pub flip_h: bool,
    pub flip_v: bool,
    pub rwal_thumb: Option<String>,
    pub rwal_clamp: Option<String>,
    pub rwal_accent: Option<u32>,
    pub rwal_order: Option<String>,
    pub get_displays: bool,
    pub get_templates: bool,
    pub get_current_colorscheme: bool,
    pub get_image_ops: bool,
    pub get_rwal_params: bool,
    pub get_config: bool,
    pub get_w_cache: bool,
    pub get_c_cache: bool,
    pub w_cache_on_miss: bool,
    pub c_cache_on_miss: bool,
}

impl Request {
    pub fn from_args(input: Vec<String>) -> Self {
        // booleans
        let w_set = input.contains(&"-S".to_string());
        let w_cache = input.contains(&"-W".to_string()) && !w_set;
        let c_set = input.contains(&"-T".to_string());
        let c_cache = input.contains(&"-C".to_string()) && !c_set;
        let cache = input.contains(&"--cache".to_string());

        let invert = input.contains(&"--invert".to_string());
        let flip_h = input.contains(&"--fliph".to_string());
        let flip_v = input.contains(&"--flipv".to_string());

        let get_displays = input.contains(&"--get-displays".to_string());
        let get_templates = input.contains(&"--get-templates".to_string());
        let get_current_colorscheme = input.contains(&"--get-current-scheme".to_string());
        let get_image_ops = input.contains(&"--get-image-ops".to_string());
        let get_rwal_params = input.contains(&"--get-rwal-params".to_string());
        let get_config = input.contains(&"--get-config".to_string());
        let get_w_cache = input.contains(&"--get-w-cache".to_string());
        let get_c_cache = input.contains(&"--get-c-cache".to_string());
        let w_cache_on_miss = input.contains(&"--w-cache-on-miss".to_string());
        let c_cache_on_miss = input.contains(&"--c-cache-on-miss".to_string());

        // strings
        let image = get_value::<String>(&input, "-I");
        let set_command = get_value::<String>(&input, "--set-command");
        let resize_alg = get_value::<String>(&input, "--resize-alg");
        let rwal_thumb = get_value::<String>(&input, "--thumb");
        let rwal_clamp = get_value::<String>(&input, "--clamp");
        let rwal_order = get_value::<String>(&input, "--order");
        // nums
        let contrast = get_value::<i32>(&input, "--contrast");
        let brightness = get_value::<f32>(&input, "--brightness");
        let hue = get_value::<i32>(&input, "--hue");
        let blur = get_value::<f32>(&input, "--blur");
        let rwal_accent = get_value::<u32>(&input, "--accent");
        // arrays
        let displays = get_displays_value(&input, "--displays");
        let templates = get_templates_value(&input, "--templates");

        Self {
            w_set,
            w_cache,
            c_set,
            c_cache,
            cache,
            image,
            set_command,
            contrast,
            brightness,
            hue,
            blur,
            invert,
            flip_h,
            flip_v,
            displays,
            templates,
            resize_alg,
            rwal_thumb,
            rwal_clamp,
            rwal_accent,
            rwal_order,
            get_displays,
            get_templates,
            get_current_colorscheme,
            get_image_ops,
            get_rwal_params,
            get_config,
            get_w_cache,
            get_c_cache,
            w_cache_on_miss,
            c_cache_on_miss,
        }
    }
}

fn get_value<T: std::str::FromStr>(list: &Vec<String>, prev_element: &str) -> Option<T> {
    for (i, el) in list.iter().enumerate() {
        if el == &prev_element {
            if i + 1 < list.len() {
                return match list[i + 1].parse::<T>() {
                    Ok(val) => Some(val),
                    Err(_) => None,
                };
            }
        }
    }
    None
}

fn get_displays_value(list: &Vec<String>, prev_element: &str) -> Option<Vec<Display>> {
    let mut displays: Vec<Display> = Vec::new();

    if let Some(raw_displays) = get_value::<String>(list, prev_element) {
        for raw_display in raw_displays.split(";") {
            let data: Vec<&str> = raw_display.split(":").collect();
            if data.len() != 5 {
                continue;
            }
            displays.push(Display::new(
                data[0].parse().unwrap_or("name".to_string()),
                data[1].parse().unwrap_or(0),
                data[2].parse().unwrap_or(0),
                data[3].parse().unwrap_or(0),
                data[4].parse().unwrap_or(0),
            ));
        }
    }

    if !displays.is_empty() {
        return Some(displays);
    }
    None
}

fn get_templates_value(list: &Vec<String>, prev_element: &str) -> Option<Vec<String>> {
    let mut templates: Vec<String> = Vec::new();

    if let Some(raw_templates) = get_value::<String>(list, prev_element) {
        templates = raw_templates.split(";").map(|x| x.to_string()).collect();
    }

    if !templates.is_empty() {
        return Some(templates);
    }
    None
}

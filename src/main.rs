use std::env;
use std::process::Command;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::error::Error;

use serde_json::Value;

struct Display {
    width: u32,
    height: u32,
    margin_left: u32,
    margin_top: u32,
    name: String,
}

struct ColorVariable {
    name: String,
    value: usize,
}

struct Template {
    temp_path: String,
    conf_path: String,
    use_quotes: bool,
    use_sharps: bool,
    opacity: String,
    command: String,
}

fn read_config() -> Value {
    let home_dir = match env::var_os("HOME") {
        Some(dir) => PathBuf::from(dir),
        _none => {
            eprintln!("Error: HOME environment variable is not set.");
            std::process::exit(1);
        }
    };

    let config_path = home_dir.join(".config/rpaper/config.json");

    let mut file = File::open(config_path).unwrap();
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).unwrap();

    let data: Value = serde_json::from_str(&json_data).unwrap();

    return data;
}

fn get_displays(config: &Value) -> Vec<Display> {
    let mut displays: Vec<Display> = Vec::new();
    for raw_display in config["displays"].as_array().unwrap() {
        displays.push(Display {
            width: raw_display["width"].as_u64().unwrap() as u32,
            height: raw_display["height"].as_u64().unwrap() as u32,
            margin_left: raw_display["margin-left"].as_u64().unwrap() as u32,
            margin_top: raw_display["margin-top"].as_u64().unwrap() as u32,
            name: String::from(raw_display["name"].as_str().unwrap()),
        })
    }
    return displays;
}

fn get_color_variables(config: &Value) -> Vec<ColorVariable> {
    let mut colors: Vec<ColorVariable> = Vec::new();
    for raw_variable in config["color_variables"].as_array().unwrap() {
        colors.push(ColorVariable {
            name: String::from(raw_variable["name"].as_str().unwrap()),
            value: raw_variable["value"].as_u64().unwrap() as usize,
        })
    }
    return colors;
}

fn get_templates(config: &Value) -> Vec<Template> {
    let mut templates: Vec<Template> = Vec::new();
    for raw_template in config["templates"].as_array().unwrap() {
        templates.push(Template {
            temp_path: String::from(raw_template["template_path"].as_str().unwrap()),
            conf_path: String::from(raw_template["config_path"].as_str().unwrap()),
            use_quotes: raw_template["use_quotes"].as_bool().unwrap(),
            use_sharps: raw_template["use_sharps"].as_bool().unwrap(),
            opacity: String::from(raw_template["opacity"].as_str().unwrap()),
            command: String::from(raw_template["command"].as_str().unwrap()),
        })
    }
    return templates;
}

fn get_wal_colors(config: &Value) -> Vec<String> { 
    let mut file = File::open(config["wal_cache"].as_str().unwrap()).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let tmp: Vec<&str> = data.split("\n").collect();
    let mut res: Vec<String> = Vec::new();

    for color in tmp {
        res.push(String::from(color));
    }

    return res;
}

fn spawn(command: String) { Command::new("bash").args(["-c", &command]).spawn().expect("Err"); }
fn start(command: String) -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("bash")
        .args(["-c", &command])
        .spawn()?;

    let status = child.wait()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("Command '{}' failed with status: {}", command, status).into())
    }
}

fn get_image_name(image_path: &str) -> &str {
    let path = Path::new(image_path);
    if let Some(file_name) = path.file_name() {
        if let Some(name) = file_name.to_str() {
            return name;
        }
    }
    return image_path;
}

fn get_cached_wallpaper_names(displays: &Vec<Display>, image_name: &str) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for display in displays {
        res.push(format!("{}.{}.{}.{}.{}-{}", display.name, display.width, display.height, display.margin_left, display.margin_top, image_name));
    }
    return res;
}

fn cache_wallpaper(image_path: &str, displays: &Vec<Display>, cached_wallpapers_path: &str) {
    println!("caching");
    let image_name: &str = get_image_name(image_path);
    let cached_wallpaper_names: Vec<String> = get_cached_wallpaper_names(&displays, image_name);

    let mut cache_is_needed: bool = false;

    for cached_wallpaper_name in &cached_wallpaper_names {
        let path = format!("{}/{}", cached_wallpapers_path, cached_wallpaper_name);
        if !Path::new(&path).exists() {
            cache_is_needed = true;
        }
    }

    if !cache_is_needed { return }
    let command = format!("rpaper_cache {}", image_path);
    start(command);
}

fn set_wallpaper(image_path: &str, displays: &Vec<Display>, cached_wallpapers_path: &str, args: &str) {
    let image_name: &str = get_image_name(image_path);
    let cached_wallpaper_names: Vec<String> = get_cached_wallpaper_names(&displays, image_name);

    for i in 0..displays.len() {
        let path = format!("{}/{}", cached_wallpapers_path, cached_wallpaper_names[i]);
        let command = format!("swww img {} {} -o {}", path, args, displays[i].name);
        if Path::new(&path).exists() {
            spawn(command)
        }
    }
}

fn templates(config: &Value) {
    let variables = get_color_variables(config);
    let templates = get_templates(config);
    let colors = get_wal_colors(config);

    for template in templates {
        let mut file = File::open(template.temp_path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        for variable in &variables {
            let mut color = format!("{}{}", &colors[variable.value], template.opacity);

            if !template.use_sharps {
                color = String::from(&color[1..]);
            }
            if template.use_quotes {
                color = format!("{}{}{}", '"', color, '"');
            }

            data = data.replace(&variable.name, &color);
        } 
        let mut file = File::create(template.conf_path).expect("Failed to create file");
        file.write_all(data.as_bytes()).expect("Failed to write to file");
        spawn(template.command);
    }
}

fn get_image_path() -> String {
    let args: Vec<String> = env::args().collect();
    let image_path: String = String::from(&args[1]);

    let current_dir = std::env::current_dir().unwrap();
    let absolute_path = current_dir.join(image_path).to_string_lossy().to_string();
    
    return absolute_path;
}

fn main() {
    let config: Value = read_config();
    let displays: Vec<Display> = get_displays(&config);
    let cached_wallpapers_path: &str = config["cached_wallpapers_dir"].as_str().unwrap();
    let raw_swww_args: &str = config["swww_params"].as_str().unwrap();
    let raw_wal_args = String::from(config["wal_params"].as_str().unwrap());
    
    let image_path: &str = &get_image_path();

    let wal_args = format!("python -m pywal {} -i {}", raw_wal_args, image_path);

    if config["use_pywal"].as_bool().unwrap() { start(wal_args); }
    if config["apply_templates"].as_bool().unwrap() { templates(&config) }

    cache_wallpaper(image_path, &displays, cached_wallpapers_path);
    set_wallpaper(image_path, &displays, cached_wallpapers_path, raw_swww_args);

}
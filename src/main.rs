use std::env;
use std::process::Command;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::error::Error;

use serde_json::Value;

mod displays;
mod templates;
mod colorvariables;


fn get_wal_colors(path: &str) -> Vec<String> {
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

fn get_cached_wallpaper_names(displays: &Vec<displays::Display>, image_name: &str) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for display in displays {
        res.push(format!("{}.{}.{}.{}.{}-{}", display.name, display.width, display.height, display.margin_left, display.margin_top, image_name));
    }
    return res;
}

fn cache_wallpaper(image_path: &str, displays: &Vec<displays::Display>, cached_wallpapers_path: &str) {
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
    let _ = start(command);
}

fn set_wallpaper(image_path: &str, displays: &Vec<displays::Display>, cached_wallpapers_path: &str, args: &str) {
    let image_name: &str = get_image_name(image_path);
    let cached_wallpaper_names: Vec<String> = get_cached_wallpaper_names(&displays, image_name);

    for i in 0..displays.len() {
        let path = format!("{}/{}", cached_wallpapers_path, cached_wallpaper_names[i]);
        let command = format!("swww img {} {} -o {}", path, args, displays[i].name);
        if Path::new(&path).exists() {
            let _ = start(command);
        }
    }
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
    return  hex;
}

fn templates(templates: Vec<templates::Template>, variables: Vec<colorvariables::ColorVariable>, wal_color_path: &str) {
    let colors = get_wal_colors(wal_color_path);

    for template in templates {
        let mut file = File::open(template.temp_path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();

        for variable in &variables {
            let value = &colors[variable.value][1..];
            let br = variable.brightness;
            let r:u8 = u8::from_str_radix(&value[0..2], 16).unwrap(); 
            let g:u8 = u8::from_str_radix(&value[2..4], 16).unwrap(); 
            let b:u8 = u8::from_str_radix(&value[4..6], 16).unwrap(); 

            let mut color = format!("#{}{}{}{}", process_color(r, br), process_color(g, br), process_color(b, br), template.opacity);

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

fn read_data(config_path: PathBuf) -> Value {
    let mut file = File::open(config_path).unwrap();
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).unwrap();

    let data: Value = serde_json::from_str(&json_data).unwrap();

    return data;
}

struct Config {
    displays: Vec<displays::Display>,
    templates: Vec<templates::Template>,
    colorvars: Vec<colorvariables::ColorVariable>,
    cached_wallpapers_path: String,
    raw_swww_args: String,
    raw_wal_args: String,
    wal_path: String,
    use_pywal: bool,
    apply_templates: bool,
}

fn main() {
    let home_dir = match env::var_os("HOME") {
        Some(dir) => PathBuf::from(dir),
        _none => {
            eprintln!("Error: HOME environment variable is not set.");
            std::process::exit(1);
        }
    };

    let config_path: PathBuf = home_dir.join(".config/rpaper/config.json");

    //todo argparser

    let data: Value = read_data(config_path);

    let config: Config = Config {
        displays: displays::get_displays(&data),
        templates: templates::get_templates(&data),
        colorvars: colorvariables::get_color_variables(&data),
        cached_wallpapers_path: String::from(data["cached_wallpapers_dir"].as_str().unwrap()),
        raw_swww_args: String::from(data["swww_params"].as_str().unwrap()),
        raw_wal_args: String::from(data["wal_params"].as_str().unwrap()),
        wal_path: String::from(data["wal_cache"].as_str().unwrap()),
        use_pywal: data["use_pywal"].as_bool().unwrap(),
        apply_templates: data["apply_templates"].as_bool().unwrap(),
        };

    let image_path: &str = &get_image_path();

    let wal_args = format!("python -m pywal {} -i {}", config.raw_wal_args, image_path);

    if config.use_pywal { 
        let _ = start(wal_args); 
    }
    if config.apply_templates { 
        templates(config.templates, config.colorvars, &config.wal_path);
    }
    
    cache_wallpaper(image_path, &config.displays, &config.cached_wallpapers_path);
    set_wallpaper(image_path, &config.displays, &config.cached_wallpapers_path, &config.raw_swww_args);
}
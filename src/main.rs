use std::env;
use std::process::Command;
use std::process::exit;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::error::Error;
use std::thread;
use image;
use image::imageops::FilterType::Triangle;
use serde_json::Value;

mod displays;
mod templates;
mod colorvariables;
mod config;


fn get_wal_colors(path: PathBuf) -> Vec<String> {
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
fn start(command: &str) -> Result<(), Box<dyn Error>> {
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

fn get_cached_wallpaper_names(displays: &Vec<displays::Display>, image_name: &str) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for display in displays {
        res.push(format!("{}.{}.{}.{}.{}-{}", display.name, display.width, display.height, display.margin_left, display.margin_top, image_name));
    }
    return res;
}

fn calculate_width_height(image_width: u32, image_height: u32, max_width: u32, max_height: u32) -> (u32, u32) {
    let w_diff: f32 = max_width as f32 / image_width as f32;

    let mut width: f32 =  image_width as f32 * w_diff;
    let mut height: f32 = image_height as f32 * w_diff;

    let h_diff: f32 = max_height as f32 / height as f32;

    if h_diff > 1.0 {
        width = width * h_diff;
        height = height * h_diff;
    }

    return (width as u32, height as u32);
}

fn get_image_name(image_path: &str) -> String {
    let path = Path::new(image_path);
    if let Some(file_name) = path.file_name() {
        if let Some(name) = file_name.to_str() {
            return String::from(name);
        }
    }
    return String::from(image_path);
}

fn cache_wallpaper(image_path: &str, displays: Vec<displays::Display>, cached_wallpapers_path: &PathBuf) -> Vec<displays::Display> {
    println!("caching");
    let image_name = get_image_name(image_path);
    let cached_wallpaper_names: Vec<String> = get_cached_wallpaper_names(&displays, &image_name);

    let displays_max_width: u32 = displays::max_width(&displays);
    let displays_max_height: u32 = displays::max_height(&displays);

    let mut threads = Vec::new();

    for (i, cached_wallpaper_name) in cached_wallpaper_names.iter().enumerate() {
        let path = format!("{}/{}", cached_wallpapers_path.display(), cached_wallpaper_name);

        if !Path::new(&path).exists() {
            println!("path not exist");
            let img_path = String::from(image_path);
            let img_name = get_image_name(image_path);
            let display = displays[i].clone();
            let thread = thread::spawn(move || {
                let mut img = image::open(img_path).unwrap();
                
                let (nw, nh) = calculate_width_height(img.width(), img.height(), displays_max_width, displays_max_height);
                
                img = img.resize(nw, nh, Triangle);
                img = img.crop_imm(display.margin_left, display.margin_top, display.width, display.height);
                let _ = img.save(config::parse_path(&format!("~/.cache/rpaper/Wallpapers/{}.{}.{}.{}.{}-{}", display.name, display.width, display.height, display.margin_left, display.margin_top, img_name)));
            });
            threads.push(thread);
        }
    }
    for thread in threads {
        thread.join().unwrap();
    }
    return displays;
}

fn set_wallpaper(image_path: &str, displays: &Vec<displays::Display>, cached_wallpapers_path: &PathBuf, command: &str) {
    let image_name = get_image_name(image_path);
    let cached_wallpaper_names: Vec<String> = get_cached_wallpaper_names(&displays, &image_name);

    for i in 0..displays.len() {
        let path = format!("{}/{}", cached_wallpapers_path.display(), cached_wallpaper_names[i]);
        let rcommand = config::parse_command(command, &path, &displays[i].name);
        if Path::new(&path).exists() {
            spawn(rcommand);
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

fn apply_templates(templates: Vec<templates::Template>, variables: Vec<colorvariables::ColorVariable>, wal_color_path: PathBuf) {
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

fn read_data(data_path: PathBuf) -> Value {
    let mut file = File::open(data_path).unwrap();
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).unwrap();

    let data: Value = serde_json::from_str(&json_data).unwrap();

    return data;
}

fn parse_args(default_config_path: PathBuf) -> (PathBuf, String) {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {exit(1)}

    let mut image_path = String::from(&args[1]);
    let current_dir = std::env::current_dir().unwrap();
    image_path = current_dir.join(image_path).to_string_lossy().to_string();
    
    let mut config_path = default_config_path;

    for (i, param) in args.iter().enumerate() {
        match param.as_str() {
            "-c" | "--conf" => {
                config_path = config::parse_path(&args[i+1])
            },
            _ => {}
        }
    }

    return (config_path, image_path);
}

fn main() {
    let mut config_path: PathBuf = config::parse_path("~/.config/rpaper/config.json");
    let image_path: String;

    (config_path, image_path) = parse_args(config_path);

    let config_data: Value = read_data(config_path);
    let config: config::Config = config::get_config(&config_data, &image_path);
    let mut displays = displays::get_displays(&config_data);

    if config.change_colorscheme { 
        let _ = start(&config.change_colors_command); 
    }
    if config.apply_templates { 
        let templates_data: Value = read_data(config.templates_path);
        let variables_data: Value = read_data(config.colorvars_path);
        
        let templates = templates::get_templates(&templates_data);
        let variables = colorvariables::get_color_variables(&variables_data);
        
        apply_templates(templates, variables, config.color_scheme_file);
    }
    if config.cache_wallpaper {
        displays = cache_wallpaper(&image_path, displays, &config.cached_wallpapers_path);
        if config.set_wallpaper {
            set_wallpaper(&image_path, &displays, &config.cached_wallpapers_path, &config.set_wallpaper_command);
        }
    }
    exit(0);
}
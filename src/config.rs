use serde_json::Value;
use std::path::PathBuf;
use std::env;

pub struct Config {
    pub templates_path: PathBuf,
    pub colorvars_path: PathBuf,
    pub cached_wallpapers_path: PathBuf,
    pub color_scheme_file: PathBuf,
    pub set_wallpaper_command: String,
    pub change_colors_command: String,
    pub change_colorscheme: bool,
    pub apply_templates: bool,
}

fn add_home_path_to_string(path: &str) -> PathBuf {
    let home_dir = match env::var_os("HOME") {
        Some(dir) => PathBuf::from(dir),
        _none => {
            eprintln!("Error: HOME environment variable is not set.");
            std::process::exit(1);
        }
    };

    let res: PathBuf = home_dir.join(path);
    return res;
}

pub fn parse_path(path: &str) -> PathBuf {
    if &path[0..1] == "~" {
        return add_home_path_to_string(&path[2..]);
    }
    return PathBuf::from(path);
}

pub fn parse_command(command: &str, image_path: &str) -> String {
    return command.replace("{image}", image_path);
}

pub fn get_config(config_data: &Value, image_path: &String) -> Config {
    let templates_path = parse_path(config_data["templates_path"].as_str().unwrap());
    let colorvars_path = parse_path(config_data["variables_path"].as_str().unwrap());
    let cached_wallpapers_path = parse_path(config_data["cached_wallpapers_dir"].as_str().unwrap());
    let color_scheme_file = parse_path(config_data["color_scheme_file"].as_str().unwrap());
    
    let set_wallpaper_command = String::from(config_data["set_wallpaper_command"].as_str().unwrap());
    let change_color_scheme_command = parse_command(config_data["change_color_scheme_command"].as_str().unwrap(), &image_path);

    let config: Config = Config {
        // Path
        templates_path: templates_path,
        colorvars_path: colorvars_path,
        cached_wallpapers_path: cached_wallpapers_path,
        color_scheme_file: color_scheme_file,
        // Command
        set_wallpaper_command: set_wallpaper_command,
        change_colors_command: change_color_scheme_command,
        // Booleans
        change_colorscheme: config_data["change_colorscheme"].as_bool().unwrap(),
        apply_templates: config_data["apply_templates"].as_bool().unwrap(),
        };
    return config;
}
use serde_json::Value;

use crate::utils::{parse_path, parse_command};

pub struct Config {
    pub templates_path: String,
    pub colorvars_path: String,
    pub color_scheme_file: String,
    pub cached_images_path: String,

    pub set_wallpaper_command: String,
    pub change_colors_command: String,
    pub wallpaper_resize_backend: String,

    pub change_colorscheme: bool,
    pub apply_templates: bool,
    pub cache_wallpaper: bool,
    pub set_wallpaper: bool,
}

pub fn get_config(config_data: &Value, image_path: &String) -> Config {
    let templates_path = parse_path(config_data["templates_path"].as_str().unwrap());
    let colorvars_path = parse_path(config_data["variables_path"].as_str().unwrap());
    let cached_images_path = parse_path(config_data["cached_wallpapers_dir"].as_str().unwrap());
    let color_scheme_file = parse_path(config_data["color_scheme_file"].as_str().unwrap());

    let set_wallpaper_command =
        String::from(config_data["set_wallpaper_command"].as_str().unwrap());
    let change_color_scheme_command = parse_command(
        config_data["change_color_scheme_command"].as_str().unwrap(),
        &image_path,
        "",
    );
    let wallpaper_resize_backend = String::from(config_data["wallpaper_resize_backend"].as_str().unwrap());

    let config: Config = Config {
        // Path
        templates_path: templates_path,
        colorvars_path: colorvars_path,
        cached_images_path: cached_images_path,
        color_scheme_file: color_scheme_file,
        // Command
        set_wallpaper_command: set_wallpaper_command,
        change_colors_command: change_color_scheme_command,
        wallpaper_resize_backend: wallpaper_resize_backend,
        // Booleans
        change_colorscheme: config_data["change_colorscheme"].as_bool().unwrap(),
        apply_templates: config_data["apply_templates"].as_bool().unwrap(),
        cache_wallpaper: config_data["cache_wallpaper"].as_bool().unwrap(),
        set_wallpaper: config_data["set_wallpaper"].as_bool().unwrap(),
    };
    return config;
}

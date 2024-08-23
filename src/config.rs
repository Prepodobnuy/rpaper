use serde_json::Value;

use crate::utils::{parse_command, parse_path};

pub struct Config {
    pub templates_path: String,
    pub colorvars_path: String,
    pub cached_images_path: String,
    pub color_scheme_file: String,

    pub set_wallpaper_command: String,
    pub change_color_scheme_command: String,
    pub wallpaper_resize_backend: String,

    pub change_colorscheme: bool,
    pub apply_templates: bool,
    pub cache_wallpaper: bool,
    pub set_wallpaper: bool,

    pub rwal_cache_dir: String,
    pub rwal_thumb: (u32, u32),
    pub rwal_accent_color: u32,
    pub rwal_clamp_min_v: f32,
    pub rwal_clamp_max_v: f32,
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

    let wallpaper_resize_backend =
        String::from(config_data["wallpaper_resize_backend"].as_str().unwrap());

    let rwal_cache_dir = parse_path(config_data["rwal_cache_dir"].as_str().unwrap());
    let rwal_thumb_w = config_data["rwal_thumb_w"].as_u64().unwrap_or(200) as u32;
    let rwal_thumb_h = config_data["rwal_thumb_h"].as_u64().unwrap_or(200) as u32;
    let rwal_thumb = (rwal_thumb_w, rwal_thumb_h);
    let rwal_accent_color = config_data["rwal_accent_color"].as_u64().unwrap_or(5) as u32;
    let rwal_clamp_min_v = config_data["rwal_clamp_min_v"].as_f64().unwrap_or(170.0) as f32;
    let rwal_clamp_max_v = config_data["rwal_clamp_max_v"].as_f64().unwrap_or(200.0) as f32;

    let config: Config = Config {
        // Path
        templates_path,
        colorvars_path,
        cached_images_path,
        color_scheme_file,
        // Command
        set_wallpaper_command,
        change_color_scheme_command,
        wallpaper_resize_backend,
        // Booleans
        change_colorscheme: config_data["change_colorscheme"].as_bool().unwrap(),
        apply_templates: config_data["apply_templates"].as_bool().unwrap(),
        cache_wallpaper: config_data["cache_wallpaper"].as_bool().unwrap(),
        set_wallpaper: config_data["set_wallpaper"].as_bool().unwrap(),
        //rwal
        rwal_cache_dir,
        rwal_thumb,
        rwal_accent_color,
        rwal_clamp_min_v,
        rwal_clamp_max_v,
    };
    return config;
}

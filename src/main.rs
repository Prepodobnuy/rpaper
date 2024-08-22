use serde_json::Value;
use wallpaper::get_cached_images_paths;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;

mod config;
mod displays;
mod templates;
mod wallpaper;
mod utils;



fn get_image_name(image_path: &str) -> String {
    let path = Path::new(image_path);
    if let Some(file_name) = path.file_name() {
        if let Some(name) = file_name.to_str() {
            return String::from(name);
        }
    }
    return String::from(image_path);
}

fn read_data(data_path: String) -> Value {
    let mut file = File::open(data_path).unwrap();
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).unwrap();

    let data: Value = serde_json::from_str(&json_data).unwrap();

    return data;
}

fn main() {
    let mut config_path: String = utils::parse_path("~/.config/rpaper/config.json");
    let image_path: String;
    let cache_only: bool;

    (config_path, image_path, cache_only) = utils::parse_args(config_path);

    let image_name = get_image_name(&image_path);

    let config_data: Value = read_data(config_path);
    let config: config::Config = config::get_config(&config_data, &image_path);
    let displays = displays::get_displays(&config_data);
    let image_operations  = wallpaper::Image_operations::new(&config_data);
    let cached_wallpapers_names = wallpaper::get_cached_images_names(&displays, &image_name, &image_operations);
    let cached_wallpapers_paths = wallpaper::get_cached_images_paths(&cached_wallpapers_names, &config.cached_images_path);
    let image_resize_algorithm = config.wallpaper_resize_backend; 

    if cache_only {
        wallpaper::cache(&image_path, &image_name, &displays, &cached_wallpapers_paths, &cached_wallpapers_names, &image_resize_algorithm, &image_operations);
        return;
    }

    if config.change_colorscheme {
        let _ = utils::start(&config.change_colors_command);
    }
    if config.apply_templates {
        let templates_value: Value = read_data(config.templates_path);
        let variables_value: Value = read_data(config.colorvars_path);

        templates::apply_templates(templates_value, variables_value, config.color_scheme_file);
    }
    if config.cache_wallpaper {
        wallpaper::cache(&image_path, &image_name, &displays, &cached_wallpapers_paths, &cached_wallpapers_names, &image_resize_algorithm, &image_operations);
        if config.set_wallpaper {
            wallpaper::set(
                &displays,
                &cached_wallpapers_paths,
                &config.set_wallpaper_command,
            );
        }
    }
    exit(0);
}

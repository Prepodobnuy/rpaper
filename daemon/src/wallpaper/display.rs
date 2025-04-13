use std::path::Path;
use std::thread;

use crate::daemon::config::Config;
use crate::wallpaper::image::ImageOperations;
use crate::{encode_string, expand_user, get_image_name, spawn, WALLPAPERS_DIR};
use common::display::Display;

use super::image::get_image;

pub fn displays_max_width(displays: &Vec<Display>) -> u32 {
    let mut res: u32 = 0;
    for display in displays {
        if display.w + display.x > res {
            res = display.w + display.x
        }
    }

    res
}

pub fn displays_max_height(displays: &Vec<Display>) -> u32 {
    let mut res: u32 = 0;
    for display in displays {
        if display.h + display.y > res {
            res = display.h + display.y
        }
    }

    res
}

pub fn calculate_width_height(
    image_width: u32,
    image_height: u32,
    max_width: u32,
    max_height: u32,
) -> (u32, u32) {
    let w_diff: f32 = max_width as f32 / image_width as f32;

    let mut width: f32 = image_width as f32 * w_diff;
    let mut height: f32 = image_height as f32 * w_diff;

    let h_diff: f32 = max_height as f32 / height as f32;

    if h_diff > 1.0 {
        width = width * h_diff;
        height = height * h_diff;
    }

    return (width as u32, height as u32);
}

fn get_file_extension(file_name: &str) -> &str {
    if let Some(pos) = file_name.rfind('.') {
        if pos != 0 {
            return &file_name[pos + 1..];
        }
    }
    ""
}

pub fn get_cached_image_names(
    displays: &Vec<Display>,
    image_ops: &ImageOperations,
    image_path: &str,
) -> Vec<String> {
    let image_name = get_image_name(image_path);
    let image_extension = get_file_extension(&image_name);
    let mut cached_images: Vec<String> = Vec::new();
    for display in displays {
        cached_images.push(format!(
            "{}.{}",
            encode_string(&format!(
                "{}{}{}{}{}{}{}{}{}{}{}{}{}",
                image_name,
                display.name,
                display.w,
                display.h,
                display.x,
                display.y,
                image_ops.contrast,
                image_ops.brightness,
                image_ops.hue,
                image_ops.blur,
                image_ops.invert,
                image_ops.flip_h,
                image_ops.flip_v,
            )),
            image_extension
        ))
    }
    cached_images
}

pub fn get_cached_image_paths(cache_images: &Vec<String>, cache_path: &str) -> Vec<String> {
    let mut cache_paths: Vec<String> = Vec::new();

    for cache_image in cache_images {
        cache_paths.push(format!("{}/{}", cache_path, cache_image,))
    }

    cache_paths
}

pub fn cache_wallpaper(config: &Config, image_path: &str) {
    if let Some(displays) = &config.displays {
        if let Some(image_ops) = &config.image_operations {
            if let Some(image_resize_algorithm) = &config.resize_algorithm {
                let cache_paths = get_cached_image_paths(
                    &get_cached_image_names(displays, image_ops, image_path),
                    WALLPAPERS_DIR,
                );

                let image = get_image(image_path, image_ops, displays, image_resize_algorithm);

                let mut handlers = Vec::new();

                for (i, cache_path) in cache_paths.into_iter().enumerate() {
                    let display = displays[i].clone();
                    let cache_path = String::from(cache_path);
                    let mut image = image.clone();
                    let thread = thread::spawn(move || {
                        image = image.crop_imm(display.x, display.y, display.w, display.h);
                        let _ = image.save(expand_user(&cache_path));
                        std::mem::drop(image);
                    });
                    handlers.push(thread)
                }

                for handler in handlers {
                    handler.join().unwrap();
                }
            }
        }
    }
}

fn parse_set_command(
    command: &str,
    image_path: &str,
    original_image_path: &str,
    display: &str,
) -> String {
    command
        .replace("{image}", &expand_user(image_path))
        .replace("{default_image}", original_image_path)
        .replace("{display}", display)
}

pub fn set_wallpaper(config: &Config, image_path: &str) {
    if let Some(displays) = &config.displays {
        if let Some(image_ops) = &config.image_operations {
            if let Some(set_command) = &config.set_command {
                let cache_paths = get_cached_image_paths(
                    &get_cached_image_names(displays, image_ops, image_path),
                    WALLPAPERS_DIR,
                );

                for cache_path in &cache_paths {
                    if !Path::new(&expand_user(cache_path)).exists() {
                        cache_wallpaper(config, image_path);
                        break;
                    }
                }

                for (i, cache_path) in cache_paths.into_iter().enumerate() {
                    let command =
                        parse_set_command(set_command, &cache_path, image_path, &displays[i].name);
                    spawn(&command);
                }
            }
        }
    }
}

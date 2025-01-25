use std::thread;
use std::path::Path;

use image::imageops::{CatmullRom, Gaussian, Lanczos3, Nearest, Triangle};
use image::{self, DynamicImage};

use crate::daemon::config::Config;
use crate::logger::logger::log;
use crate::{encode_string, expand_user, get_image_name, spawn, WALLPAPERS_DIR};

#[derive(Clone)]
pub struct Display {
    w: u32,
    h: u32,
    x: u32,
    y: u32,
    name: String,
}

impl Display {
    pub fn new(
        w: u32,
        h: u32,
        x: u32,
        y: u32,
        name: String,
    ) -> Self {
        Display {
            w,
            h,
            x,
            y,
            name,
        }
    }

    pub fn name(&self) -> String {self.name.clone()}
    pub fn width(&self) -> u32 {self.w}
    pub fn height(&self) -> u32 {self.h}
    pub fn x(&self) -> u32 {self.x}
    pub fn y(&self) -> u32 {self.y}

    pub fn from_string(string: &str) -> Result<Display, String> {
        // create Display from string
        // Example:
        // input: HDMI-A-1:1920:1080:0:0
        // output: Display {1920, 1080, 0, 0, HDMI-A-1}
        
        let mut name: String = String::new();
        let mut w: u32 = 0;
        let mut h: u32 = 0;
        let mut x: u32 = 0;
        let mut y: u32 = 0;

        if string.split(":").collect::<Vec<_>>().len() != 5 {
            return Err(String::from("Unable to parse string"));
        }

        string.split(":")
            .enumerate()
            .for_each(|(i, param)| {
                match i {
                    0 => {name = String::from(param)},
                    1 => {w = param.parse().unwrap_or(0)},
                    2 => {h = param.parse().unwrap_or(0)},
                    3 => {x = param.parse().unwrap_or(0)},
                    4 => {y = param.parse().unwrap_or(0)},
                    _ => {},
                }
            });

        Ok(Display{
            w,
            h,
            x,
            y,
            name,
        })
    }
}

fn displays_max_width(displays: &Vec<Display>) -> u32 {
    let mut res: u32 = 0;
    for display in displays {
        if display.w + display.x > res {
            res = display.w + display.x
        }
    }

    res
}

fn displays_max_height(displays: &Vec<Display>) -> u32 {
    let mut res: u32 = 0;
    for display in displays {
        if display.h + display.y > res {
            res = display.h + display.y
        }
    }

    res
}

#[derive(Clone)]
pub struct WCacheInfo {
    pub display_name: String,
    pub path: String,
}

impl WCacheInfo {
    pub fn new(display_name: &str, path: &str) -> Self {
        Self {
            display_name: display_name.to_string(),
            path: path.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ImageOperations {
    pub contrast: f32,
    pub brightness: i32,
    pub hue: i32,
    pub blur: f32,
    pub invert: bool,
    pub flip_h: bool,
    pub flip_v: bool,
}

impl ImageOperations {
    pub fn new(
        contrast: f32,
        brightness: i32,
        hue: i32,
        blur: f32,
        invert: bool,
        flip_h: bool,
        flip_v: bool,
    ) -> Self {
        ImageOperations {
            contrast,
            brightness,
            hue,
            blur,
            invert,
            flip_h,
            flip_v,
        }       
    }
}

fn calculate_width_height(
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

fn get_image(
    img_path: &str,
    image_ops: &ImageOperations,
    displays: &Vec<Display>,
    image_resize_algorithm: &str,
) -> DynamicImage {
    let displays_max_width: u32 = displays_max_width(&displays);
    let displays_max_height: u32 = displays_max_height(&displays);

    let img_ra = match image_resize_algorithm {
        "Nearest" => Nearest,
        "CatmullRom" => CatmullRom,
        "Gaussian" => Gaussian,
        "Lanczos3" => Lanczos3,
        _ => Triangle,
    };

    if let Ok(mut _image) = image::open(img_path) {
        let (nw, nh) = calculate_width_height(
            _image.width(),
            _image.height(),
            displays_max_width,
            displays_max_height,
        );
        _image = _image.resize(nw, nh, img_ra);
        if image_ops.contrast != 0.0 {
            _image = _image.adjust_contrast(image_ops.contrast)
        }
        if image_ops.brightness != 0 {
            _image = _image.brighten(image_ops.brightness)
        }
        if image_ops.hue != 0 {
            _image = _image.huerotate(image_ops.hue)
        }
        if image_ops.blur != 0.0 {
            _image = _image.blur(image_ops.blur)
        }
        if image_ops.flip_h {
            _image = _image.fliph()
        }
        if image_ops.flip_v {
            _image = _image.flipv()
        }
        if image_ops.invert {
            _image.invert()
        }
        _image
    } else {
        DynamicImage::new(1000, 1000, image::ColorType::Rgb8)
    }
}

fn get_file_extension(file_name: &str) -> &str {
    if let Some(pos) = file_name.rfind('.') {
        if pos != 0 {
            return &file_name[pos + 1..]
        }
    }
    ""
}

pub fn get_cached_image_names(displays: &Vec<Display>, image_ops: &ImageOperations, image_path: &str) -> Vec<String> {
    let image_name = get_image_name(image_path);
    let image_extension = get_file_extension(&image_name);
    let mut cached_images: Vec<String> = Vec::new();
    for display in displays {
        cached_images.push(
            format!(
                "{}.{}",
                encode_string(
                    &format!(
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
                    )
                ),
                image_extension
            )
        )
    }
    cached_images
}

pub fn get_cached_image_paths(cache_images: &Vec<String>, cache_path: &str) -> Vec<String> {
    let mut cache_paths: Vec<String> = Vec::new();

    for cache_image in cache_images {
        cache_paths.push(
            format!(
                "{}/{}",
                cache_path,
                cache_image,
            )
        )
    }

    cache_paths
}

pub fn cache_wallpaper(config: &Config, image_path: &str) {
    if let Some(displays) = &config.displays {
        if let Some(image_ops) = &config.image_operations {
            if let Some(image_resize_algorithm) = &config.resize_algorithm {
                log("Caching wallpaper...");
                let cache_paths = get_cached_image_paths(
                    &get_cached_image_names(displays, image_ops, image_path), 
                    WALLPAPERS_DIR
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

fn parse_set_command(command: &str, image_path: &str, original_image_path: &str, display: &str) -> String {
    command
        .replace("{image}", image_path)
        .replace("{default_image}", original_image_path)
        .replace("{display}", display)
}

pub fn set_wallpaper(config: &Config, image_path: &str) {
    if let Some(displays) = &config.displays {
        if let Some(image_ops) = &config.image_operations {
            if let Some(set_command) = &config.wallpaper_set_command {
                let cache_paths = get_cached_image_paths(
                    &get_cached_image_names(displays, image_ops, image_path), 
                    WALLPAPERS_DIR
                );
            
                for cache_path in &cache_paths {
                    if !Path::new(&expand_user(cache_path)).exists() {
                        cache_wallpaper(config, image_path);
                        break;
                    }
                }

                log("Setting wallpaper...");
            
                for (i, cache_path) in cache_paths.into_iter().enumerate() {
                    let command = parse_set_command(set_command, &cache_path, image_path, &displays[i].name);
                    spawn(&command);
                }
            }
        }
    }
}
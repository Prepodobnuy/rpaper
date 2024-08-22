use std::thread;
use std::path::Path;
use image;
use image::imageops::{Nearest, Triangle, CatmullRom, Gaussian, Lanczos3};
use serde_json::Value;
use crate::displays;
use crate::utils::{parse_path, parse_command, spawn};

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

pub struct Image_operations {
    change_contrast: bool,
    change_brightness: bool,
    change_huerotate: bool,
    change_blur: bool,
    image_flip_h: bool,
    image_flip_v: bool,
    contrast: f32,
    brightness: i32,
    huerotate: i32,
    blur: f32,
} 

impl Image_operations {
    pub fn new(config_data: &Value) -> Self {
        let change_contrast = config_data["change_contrast"].as_bool().unwrap_or(false);
        let change_brightness = config_data["change_brightness"].as_bool().unwrap_or(false);
        let change_huerotate = config_data["change_huerotate"].as_bool().unwrap_or(false);
        let change_blur = config_data["change_blur"].as_bool().unwrap_or(false);
        let image_flip_h = config_data["image_flip_h"].as_bool().unwrap_or(false);
        let image_flip_v = config_data["image_flip_v"].as_bool().unwrap_or(false);
        let contrast = config_data["contrast"].as_f64().unwrap_or(0.0) as f32;
        let brightness = config_data["brightness"].as_i64().unwrap_or(0) as i32;
        let huerotate = config_data["huerotate"].as_i64().unwrap_or(0) as i32;
        let blur = config_data["blur"].as_f64().unwrap_or(0.0) as f32;

        Image_operations {
            change_contrast,
            change_brightness,
            change_huerotate,
            change_blur,
            image_flip_h,
            image_flip_v,
            contrast,
            brightness,
            huerotate,
            blur,
        }
    }

    pub fn clone(&self) -> Image_operations {
        Image_operations {
            change_contrast: self.change_contrast,
            change_brightness: self.change_brightness,
            change_huerotate: self.change_huerotate,
            change_blur: self.change_blur,
            image_flip_h: self.image_flip_h,
            image_flip_v: self.image_flip_v,
            contrast: self.contrast,
            brightness: self.brightness,
            huerotate: self.huerotate,
            blur: self.blur,
        }
    }
}

pub fn get_cached_images_names(displays: &Vec<displays::Display>, image_name: &str, image_ops: &Image_operations) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for display in displays {
        let mut image_name = format!("{}.{}.{}.{}.{}-{}",
            display.name,
            display.width,
            display.height,
            display.margin_left,
            display.margin_top,
            image_name
        );

        if image_ops.change_contrast  {image_name = format!("CR{}{}", image_ops.contrast, image_name)}
        if image_ops.change_brightness{image_name = format!("BR{}{}", image_ops.brightness, image_name)}
        if image_ops.change_huerotate {image_name = format!("HUE{}{}", image_ops.huerotate, image_name)}
        if image_ops.change_blur      {image_name = format!("BLUR{}{}", image_ops.blur, image_name)}
        if image_ops.image_flip_h     {image_name = format!("H_FL{}", image_name)}
        if image_ops.image_flip_v     {image_name = format!("V_FL{}", image_name)}

        res.push(image_name);
    }
    return res;
}

pub fn get_cached_images_paths(cached_wallpapers_names: &Vec<String>, cached_wallpapers_path: &str) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for w_name in cached_wallpapers_names {
        res.push(format!("{}/{}", cached_wallpapers_path, w_name));
    }
    return res;
}

pub fn cache(
    image_path: &str,
    image_name: &str,
    displays: &Vec<displays::Display>,
    cached_wallpapers_paths: &Vec<String>,
    cached_wallpapers_names: &Vec<String>,
    image_resize_algorithm: &str,
    image_ops: &Image_operations,
) {
    let displays_max_width: u32 = displays::max_width(&displays);
    let displays_max_height: u32 = displays::max_height(&displays);

    let mut threads = Vec::new();

    for (i, path) in cached_wallpapers_paths.iter().enumerate() {

        if !Path::new(&path).exists() {
            println!("caching {} to {}", image_name, displays[i].name);
            let img_path = String::from(image_path);
            let img_ra = match image_resize_algorithm {
                "Nearest" => Nearest,
                "CatmullRom" => CatmullRom,
                "Gaussian" => Gaussian,
                "Lanczos3" => Lanczos3,
                _ => Triangle,
            };
            let img_ops = image_ops.clone();
            let display = displays[i].clone();
            let wallpaper_name = cached_wallpapers_names[i].clone();
            let thread = thread::spawn(move || {
                let mut img = image::open(img_path).unwrap();

                let (nw, nh) = calculate_width_height(
                    img.width(),
                    img.height(),
                    displays_max_width,
                    displays_max_height,
                );

                img = img.resize(nw, nh, img_ra);

                if img_ops.change_contrast   {img = img.adjust_contrast(img_ops.contrast)}
                if img_ops.change_brightness {img = img.brighten(img_ops.brightness)}
                if img_ops.change_huerotate  {img = img.huerotate(img_ops.huerotate)}
                if img_ops.change_blur       {img = img.blur(img_ops.blur)}
                if img_ops.image_flip_h      {img = img.fliph()}
                if img_ops.image_flip_v      {img = img.flipv()}

                img = img.crop_imm(
                    display.margin_left,
                    display.margin_top,
                    display.width,
                    display.height,
                );
                let _ = img.save(parse_path(&format!(
                    "~/.cache/rpaper/Wallpapers/{}",
                    wallpaper_name
                )));
            });
            threads.push(thread);
        }
    }
    for thread in threads {
        thread.join().unwrap();
    }
}


pub fn set(
    displays: &Vec<displays::Display>,
    cached_images_paths: &Vec<String>,
    command: &str,
) {
    for i in 0..displays.len() {
        let path = &cached_images_paths[i];

        let rcommand = parse_command(command, &path, &displays[i].name);
        if Path::new(&path).exists() {
            spawn(rcommand);
        }
    }
}

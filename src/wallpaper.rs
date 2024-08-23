use crate::config::ImageOperations;
use crate::displays::{self, displays_max_height, displays_max_width, Display};
use crate::utils::{get_img_ops_affected_name, parse_command, parse_path, spawn};
use image::imageops::{CatmullRom, Gaussian, Lanczos3, Nearest, Triangle};
use image::{self, DynamicImage};
use std::path::Path;
use std::thread;

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

pub fn get_image(
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

    let mut _image = image::open(img_path).unwrap();
    let (nw, nh) = calculate_width_height(
        _image.width(),
        _image.height(),
        displays_max_width,
        displays_max_height,
    );
    _image = _image.resize(nw, nh, img_ra);
    if image_ops.change_contrast {
        _image = _image.adjust_contrast(image_ops.contrast)
    }
    if image_ops.change_brightness {
        _image = _image.brighten(image_ops.brightness)
    }
    if image_ops.change_huerotate {
        _image = _image.huerotate(image_ops.huerotate)
    }
    if image_ops.change_blur {
        _image = _image.blur(image_ops.blur)
    }
    if image_ops.image_flip_h {
        _image = _image.fliph()
    }
    if image_ops.image_flip_v {
        _image = _image.flipv()
    }
    if image_ops.invert_image {
        _image.invert()
    }
    _image
}

pub fn get_cached_images_names(
    displays: &Vec<displays::Display>,
    image_name: &str,
    image_ops: &ImageOperations,
) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for display in displays {
        let mut image_name = format!(
            "{}.{}.{}.{}.{}-{}",
            display.name, display.w, display.h, display.x, display.y, image_name
        );
        image_name = get_img_ops_affected_name(&image_name, image_ops);

        res.push(image_name);
    }
    return res;
}

pub fn get_cached_images_paths(
    cached_wallpapers_names: &Vec<String>,
    cached_wallpapers_path: &str,
) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for w_name in cached_wallpapers_names {
        res.push(format!("{}/{}", cached_wallpapers_path, w_name));
    }
    return res;
}

pub fn cache(
    image_path: &str,
    image_name: &str,
    image_ops: &ImageOperations,
    image_resize_algorithm: &str,
    displays: &Vec<displays::Display>,
    cached_wallpapers_paths: &Vec<String>,
) {
    let mut threads = Vec::new();

    for (i, path) in cached_wallpapers_paths.iter().enumerate() {
        if !Path::new(&path).exists() {
            println!("caching {} to {}", image_name, displays[i].name);
            let display = displays[i].clone();
            let img_path = String::from(image_path);
            let displays = displays.clone();
            let image_ops = image_ops.clone();
            let img_resize_algorithm = String::from(image_resize_algorithm);
            let wallpaper_path = cached_wallpapers_paths[i].clone();
            let thread = thread::spawn(move || {
                let mut img = get_image(&img_path, &image_ops, &displays, &img_resize_algorithm);
                img = img.crop_imm(display.x, display.y, display.w, display.h);
                let _ = img.save(parse_path(&format!("{}", wallpaper_path)));
            });
            threads.push(thread);
        }
    }
    for thread in threads {
        thread.join().unwrap();
    }
}

pub fn set(displays: &Vec<displays::Display>, cached_images_paths: &Vec<String>, command: &str) {
    for i in 0..displays.len() {
        let path = &cached_images_paths[i];

        let rcommand = parse_command(command, &path, &displays[i].name);
        if Path::new(&path).exists() {
            spawn(rcommand);
        }
    }
}

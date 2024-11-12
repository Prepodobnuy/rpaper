use std::path::Path;
use std::thread;

use image::imageops::{CatmullRom, Gaussian, Lanczos3, Nearest, Triangle};
use image::{self, DynamicImage, RgbImage};

use crate::displays::{self, displays_max_height, displays_max_width, Display};
use crate::utils::config::ImageOperations;
use crate::utils::command::{spawn, parse_command};


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
        _image = _image.brighten(image_ops.brightness as i32) // TODO fix this heck, britness must be an i32 not f32
    }
    if image_ops.change_huerotate {
        _image = _image.huerotate(image_ops.huerotate)
    }
    if image_ops.change_blur {
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
}

pub fn get_thumbed_image(image_path: &str, image_ops: &ImageOperations, w: u32, h: u32) -> RgbImage {
    let mut _image = image::open(image_path).unwrap();
    _image = _image.resize_exact(w, h, Nearest);
    if image_ops.change_contrast {
        _image = _image.adjust_contrast(image_ops.contrast)
    }
    if image_ops.change_brightness {
        _image = _image.brighten(image_ops.brightness as i32) // 61 line lol
    }
    if image_ops.change_huerotate {
        _image = _image.huerotate(image_ops.huerotate)
    }
    if image_ops.change_blur {
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
    _image.to_rgb8()
}

pub fn cache(
    image_name: &str,
    image_path: &str,
    cache_paths: &Vec<String>,
    image_ops: &ImageOperations,
    image_resize_algorithm: &str,
    displays: &Vec<displays::Display>,
) {
    let mut threads = Vec::new();

    for (i, path) in cache_paths.iter().enumerate() {
        if !Path::new(&path).exists() {
            println!("caching {} to {}", image_name, displays[i].name);
            let display = displays[i].clone();
            let img_path = String::from(image_path);
            let displays = displays.clone();
            let image_ops = image_ops.clone();
            let img_resize_algorithm = String::from(image_resize_algorithm);
            let path = path.clone();
            let thread = thread::spawn(move || {
                let mut img = get_image(&img_path, &image_ops, &displays, &img_resize_algorithm);
                img = img.crop_imm(display.x, display.y, display.w, display.h);
                let _ = img.save(path);
            });
            threads.push(thread);
        }
    }
    for thread in threads {
        thread.join().unwrap();
    }
}

pub fn set(displays: &Vec<displays::Display>, cache_paths: &Vec<String>, image_path: &str, command: &str) {
    for i in 0..displays.len() {
        let path = &cache_paths[i];

        let rcommand = parse_command(command, &path, image_path, &displays[i].name,);
        if Path::new(&path).exists() {
            spawn(&rcommand);
        }
    }
}

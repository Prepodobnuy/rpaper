use std::thread;
use std::path::Path;
use image;
use image::imageops::{Nearest, Triangle, CatmullRom, Gaussian, Lanczos3};
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

pub fn cache(
    image_path: &str,
    image_name: &str,
    displays: &Vec<displays::Display>,
    cached_images_paths: &Vec<String>,
    image_op: &str,
) {
    let displays_max_width: u32 = displays::max_width(&displays);
    let displays_max_height: u32 = displays::max_height(&displays);

    let mut threads = Vec::new();

    for (i, path) in cached_images_paths.iter().enumerate() {

        if !Path::new(&path).exists() {
            println!("caching {} to {}", image_name, displays[i].name);
            let img_path = String::from(image_path);
            let img_name = String::from(image_name);
            let img_op = match image_op {
                "Nearest" => Nearest,
                "CatmullRom" => CatmullRom,
                "Gaussian" => Gaussian,
                "Lanczos3" => Lanczos3,
                _ => Triangle,
            };
            let display = displays[i].clone();
            let thread = thread::spawn(move || {
                let mut img = image::open(img_path).unwrap();

                let (nw, nh) = calculate_width_height(
                    img.width(),
                    img.height(),
                    displays_max_width,
                    displays_max_height,
                );

                img = img.resize(nw, nh, img_op);
                img = img.crop_imm(
                    display.margin_left,
                    display.margin_top,
                    display.width,
                    display.height,
                );
                let _ = img.save(parse_path(&format!(
                    "~/.cache/rpaper/Wallpapers/{}.{}.{}.{}.{}-{}",
                    display.name,
                    display.width,
                    display.height,
                    display.margin_left,
                    display.margin_top,
                    img_name
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

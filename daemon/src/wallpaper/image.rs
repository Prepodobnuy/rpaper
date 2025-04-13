use image::{DynamicImage, RgbImage};
use image::imageops::{CatmullRom, Gaussian, Lanczos3, Nearest, Triangle};
use serde::{Deserialize, Serialize};
use common::display::Display;

use super::display::{calculate_width_height, displays_max_height, displays_max_width};

pub fn apply_image_ops(mut image: DynamicImage, image_ops: &ImageOperations) -> DynamicImage {
    if image_ops.contrast != 0.0 {
        image = image.adjust_contrast(image_ops.contrast)
    }
    if image_ops.brightness != 0 {
        image = image.brighten(image_ops.brightness)
    }
    if image_ops.hue != 0 {
        image = image.huerotate(image_ops.hue)
    }
    if image_ops.blur != 0.0 {
        image = image.blur(image_ops.blur)
    }
    if image_ops.flip_h {
        image = image.fliph()
    }
    if image_ops.flip_v {
        image = image.flipv()
    }
    if image_ops.invert {
        image.invert()
    }
    image
}

pub fn get_thumbed_image(image_path: &str, image_ops: &ImageOperations, w: u32, h: u32) -> RgbImage {
    if let Ok(mut image) = image::open(image_path) {
        image = image.resize_exact(w, h, Nearest);
        image = apply_image_ops(image, image_ops);
        image.to_rgb8()
    } else {
        RgbImage::new(4, 4)
    }
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

    if let Ok(mut image) = image::open(img_path) {
        let (nw, nh) = calculate_width_height(
            image.width(),
            image.height(),
            displays_max_width,
            displays_max_height,
        );
        image = image.resize(nw, nh, img_ra);
        image = apply_image_ops(image, image_ops);
        image
    } else {
        DynamicImage::new(1000, 1000, image::ColorType::Rgb8)
    }
}


#[derive(Clone, Serialize, Deserialize)]
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

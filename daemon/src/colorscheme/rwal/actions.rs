use std::fs;
use std::path::Path;

use image::imageops::FilterType::Nearest;
use image::RgbImage;

use crate::expand_user;
use crate::wallpaper::display::ImageOperations;
use crate::COLORS_PATH;

use super::rwal::get_palette;
use super::rwal_params::RwalParams;

pub fn run_rwal(
    image_path: &str,
    color_scheme_path: &str,
    rwal_params: &RwalParams,
    image_ops: &ImageOperations,
) -> Vec<String> {
    if !Path::new(color_scheme_path).exists() {
        cache_rwal(image_path, color_scheme_path, rwal_params, image_ops);
    }

    if let Ok(colors) = fs::read_to_string(color_scheme_path) {
        let _ = fs::write(expand_user(COLORS_PATH), &colors);
    }

    if let Ok(colors) = fs::read_to_string(expand_user(COLORS_PATH)) {
        return colors.lines().map(|line| line.to_string()).collect();
    } else {
        Vec::new()
    }
}

pub fn cache_rwal(
    image_path: &str,
    color_scheme_path: &str,
    rwal_params: &RwalParams,
    image_ops: &ImageOperations,
) {
    let image = &get_thumbed_image(
        image_path,
        image_ops,
        rwal_params.thumb_range.0,
        rwal_params.thumb_range.1,
    );

    let pallete = get_palette(
        image,
        rwal_params.accent_color,
        rwal_params.clamp_range,
        rwal_params.order,
    )
    .join("\n");

    fs::write(color_scheme_path, &pallete).unwrap();
    fs::write(expand_user(COLORS_PATH), &pallete).unwrap();
}

fn get_thumbed_image(image_path: &str, image_ops: &ImageOperations, w: u32, h: u32) -> RgbImage {
    if let Ok(mut _image) = image::open(image_path) {
        _image = _image.resize_exact(w, h, Nearest);
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
        _image.to_rgb8()
    } else {
        RgbImage::new(4, 4)
    }
}

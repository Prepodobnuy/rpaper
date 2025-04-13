use std::fs;
use std::path::Path;

use crate::expand_user;
use crate::wallpaper::image::ImageOperations;
use crate::wallpaper::image::get_thumbed_image;
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

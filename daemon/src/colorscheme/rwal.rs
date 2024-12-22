use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use image::imageops::FilterType::Nearest;
use image::{self, RgbImage};
use kmeans_colors::{get_kmeans, Kmeans};
use palette::cast::from_component_slice;
use palette::{IntoColor, Lab, Srgb};
use color_space::{Hsv as HSV, Lab as LAB, Rgb as RGB};

use crate::wallpaper::display::ImageOperations;
use crate::{expand_user, COLORS_DIR, COLORS_PATH};

#[derive(Clone)]
pub struct RwalParams {
    pub thumb_range: (u32, u32),
    pub clamp_range: (f32, f32),
    pub accent_color: u32,
    pub colors: u32,
}

impl RwalParams {
    pub fn new(
        thumb_range: (u32, u32),
        clamp_range: (f32, f32),
        accent_color: u32,
        colors: u32,
    ) -> Self {
        RwalParams {
            thumb_range,
            clamp_range,
            accent_color,
            colors,
        }
    }
}

fn get_thumbed_image(image_path: &str, image_ops: &ImageOperations, w: u32, h: u32) -> RgbImage {
    let mut _image = image::open(image_path).unwrap();
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
        _image = _image.blur(image_ops.blur) // sigma
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

pub fn run_rwal(image_path: &str, color_scheme_path: &str, rwal_params: &RwalParams, image_ops: &ImageOperations) -> Vec<String> {
    if !Path::new(color_scheme_path).exists() {
        cache_rwal(image_path, color_scheme_path, rwal_params, image_ops);
    }

    if let Ok(colors) = fs::read_to_string(color_scheme_path) {
        let _ = fs::write(expand_user(COLORS_PATH), &colors);
    }

    if let Ok(colors) = fs::read_to_string(expand_user(COLORS_PATH)) {
        return colors.lines().map(|line| line.to_string()).collect();
    }
    else {
        Vec::new()
    }
}

pub fn cache_rwal(image_path: &str, color_scheme_path: &str, rwal_params: &RwalParams, image_ops: &ImageOperations) {
    let image = &get_thumbed_image(
        image_path, 
        image_ops, 
        rwal_params.thumb_range.0, 
        rwal_params.thumb_range.1,
    );
    
    let pallete = get_pallete(
        image, 
        rwal_params.accent_color,
        rwal_params.clamp_range,
    ).join("\n");

    fs::write(color_scheme_path, &pallete).unwrap();
    fs::write(expand_user(COLORS_PATH), &pallete).unwrap();
}

fn get_colors(image: &RgbImage) -> Vec<RGB> {
    let mut colors: Vec<RGB> = Vec::new();

    for pixel in image.pixels() {
        colors.push(RGB::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64));
    }

    colors
}

fn clamp_color(color: RGB, min_v: f32, max_v: f32) -> Vec<u8> {
    let mut hsv = HSV::from(color);
    let (h, s, v) = (hsv.h, hsv.s, hsv.v);
    let v = f32::min(f32::max(min_v / 255.0, v as f32), max_v / 255.0);
    hsv.h = h;
    hsv.s = s;
    hsv.v = v as f64;
    let rgb = RGB::from(hsv);
    let vec = vec![rgb.r as u8, rgb.g as u8, rgb.b as u8];

    vec
}

fn lab_to_hsv(lab: Lab) -> HSV {
    let l = lab.l;
    let a = lab.a;
    let b = lab.b;

    let _lab = LAB::new(l as f64, a as f64, b as f64);

    HSV::from(_lab)
}

fn merge_rgb(f_color: RGB, s_color: RGB) -> RGB {
    let r = (f_color.r + f_color.r + f_color.r + f_color.r + s_color.r) / 5.0;
    let g = (f_color.g + f_color.g + f_color.g + f_color.g + s_color.g) / 5.0;
    let b = (f_color.b + f_color.b + f_color.b + f_color.b + s_color.b) / 5.0;

    RGB::new(r, g, b)
}

fn order_colors_by_hue(clusters: Kmeans<Lab>, accent_color: u32) -> Vec<String> {
    let lab_centroids: Vec<Lab> = clusters.centroids.to_vec();
    let mut hsv_colors = Vec::new();
    for lab in lab_centroids {
        hsv_colors.push(lab_to_hsv(lab))
    }

    hsv_colors.sort_by(|a, b| a.h.partial_cmp(&b.h).unwrap_or(Ordering::Equal));

    let mut rgb_colors = Vec::new();
    for hsv in hsv_colors {
        rgb_colors.push(RGB::from(hsv));
    }

    if rgb_colors.len() < 6 {
        println!("The number of generated colors is not enough! \nNeeded colors: 5\nGenerated colors: {}", rgb_colors.len());
        for _ in 0..5 - rgb_colors.len() {
            rgb_colors.push(RGB::new(100.0, 100.0, 100.0));
        }
    }

    let accent = rgb_colors[accent_color as usize];
    let bg_color = merge_rgb(RGB::new(0.0, 0.0, 0.0), accent.clone());
    let fg_color = merge_rgb(RGB::new(255.0, 255.0, 255.0), accent.clone());

    rgb_colors.insert(0, bg_color);
    rgb_colors.push(fg_color);

    let mut res = Vec::new();

    for rgb in &rgb_colors {
        let mut _tmp = String::new();
        let mut h_r = format!("{:X}", rgb.r as u8);
        let mut h_g = format!("{:X}", rgb.g as u8);
        let mut h_b = format!("{:X}", rgb.b as u8);

        if h_r.len() == 1 {
            h_r = format!("0{}", h_r)
        }
        if h_g.len() == 1 {
            h_g = format!("0{}", h_g)
        }
        if h_b.len() == 1 {
            h_b = format!("0{}", h_b)
        }

        _tmp = format!("#{}{}{}", h_r, h_g, h_b);
        res.push(_tmp)
    }
    
    res.extend(res.clone());
    
    res
}

fn get_pallete(image: &RgbImage, accent_color: u32, clamp_range: (f32, f32)) -> Vec<String> {
    let colors = get_colors(image);
    let mut clamped_colors = Vec::new();

    for color in &colors {
        clamped_colors.extend(clamp_color(
            color.clone(), 
            clamp_range.0, 
            clamp_range.1,
        ))
    }

    let colors_slice = clamped_colors.as_slice();
    let lab: Vec<Lab> = from_component_slice::<Srgb<u8>>(&colors_slice)
        .iter()
        .map(|x| x.into_format().into_color())
        .collect();

    let mut clusters = Kmeans::new();
    for i in 0..3 {
        let run_result = get_kmeans(6, 100, 0.001, false, &lab, 64 + i as u64);
        if run_result.score < clusters.score {
            clusters = run_result;
        }
    }

    order_colors_by_hue(clusters, accent_color)
}
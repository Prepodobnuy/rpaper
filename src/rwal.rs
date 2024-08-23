use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use image;
use image::imageops::Nearest;
use image::DynamicImage;

use kmeans_colors::{get_kmeans, Kmeans};
use palette::cast::from_component_slice;
use palette::{IntoColor, Lab, Srgb};

use color_space::{Hsv as HSV, Lab as LAB, Rgb as RGB};

#[derive(Debug)]
enum Error {
    TooLarge,
}

type PalletteReadResult<T, E> = Result<T, E>;

pub struct Rwal {
    image_name: String,
    cache_dir: String,
    thumb_size: (u32, u32),
    accent_color: u32,
    clamp_min_v: f32,
    clamp_max_v: f32,
}

impl Rwal {
    pub fn new(
        image_name: &str,
        cache_dir: &str,
        thumb_size: (u32, u32),
        accent_color: u32,
        clamp_min_v: f32,
        clamp_max_v: f32,
    ) -> Self {
        Rwal {
            image_name: image_name.to_string(),
            cache_dir: cache_dir.to_string(),
            thumb_size,
            accent_color,
            clamp_min_v,
            clamp_max_v,
        }
    }

    pub fn get_pallete_cache_path(&self) -> String {
        format!(
            "{}/{}{}{}{}{}{}",
            self.cache_dir,
            self.image_name,
            self.thumb_size.0,
            self.thumb_size.1,
            self.accent_color,
            self.clamp_min_v,
            self.clamp_max_v
        )
    }

    fn is_cached(&self) -> bool {
        let cache_path = self.get_pallete_cache_path();

        Path::new(&cache_path).exists()
    }

    fn read_from_cache(&self) -> String {
        let cache_path = self.get_pallete_cache_path();

        fs::read_to_string(cache_path).unwrap()
    }

    fn cache(&self, pallete: &str) {
        let cache_path = self.get_pallete_cache_path();

        fs::File::create(&cache_path).unwrap();
        fs::write(cache_path, pallete).unwrap();
    }

    fn write_to_colors_file(&self, pallete: &str) {
        let path = format!("{}/colors", self.cache_dir);

        fs::File::create(&path).unwrap();
        fs::write(&path, pallete).unwrap();
    }

    pub fn run(&self, image: &DynamicImage) {
        if self.is_cached() {
            let pallete = self.read_from_cache();
            self.write_to_colors_file(&pallete);
        } else {
            let pallete = get_pallete(
                image,
                self.thumb_size,
                self.accent_color,
                self.clamp_min_v,
                self.clamp_max_v,
            )
            .unwrap()
            .join("\n");
            self.cache(&pallete);
            self.write_to_colors_file(&pallete);
        }
    }
}

fn get_colors(image: &DynamicImage, thumb_size: (u32, u32)) -> Vec<RGB> {
    let thumb_image = image
        .resize_exact(thumb_size.0, thumb_size.1, Nearest)
        .to_rgb8();
    let mut colors: Vec<RGB> = Vec::new();

    for pixel in thumb_image.pixels() {
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
    return vec;
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

    let accent = rgb_colors[accent_color as usize];
    let bg_color = merge_rgb(RGB::new(0.0, 0.0, 0.0), accent.clone());
    let fg_color = merge_rgb(RGB::new(255.0, 255.0, 255.0), accent.clone());

    rgb_colors.insert(0, bg_color);
    rgb_colors.push(fg_color);

    let mut res = Vec::new();

    for rgb in &rgb_colors {
        let mut tmp = String::new();
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

        tmp = format!("#{}{}{}", h_r, h_g, h_b);
        res.push(tmp)
    }
    let sres = res.clone();
    res.extend(sres);

    res
}

fn get_pallete(
    image: &DynamicImage,
    thumb_size: (u32, u32),
    accent_color: u32,
    min_v: f32,
    max_v: f32,
) -> PalletteReadResult<Vec<String>, Error> {
    let colors = get_colors(image, thumb_size);
    let mut clamped_colors = Vec::new();

    for color in &colors {
        clamped_colors.extend(clamp_color(color.clone(), min_v, max_v))
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

    let result = order_colors_by_hue(clusters, accent_color);

    return Ok(result);
}

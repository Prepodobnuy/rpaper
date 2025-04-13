use std::cmp::Ordering;

use image::RgbImage;
use kmeans_colors::{get_kmeans, Kmeans};

use color_space::{FromRgb, Hsv, Lab as CSLab, Rgb, ToRgb};
use palette::cast::from_component_slice;
use palette::{IntoColor, Lab, Srgb};

use crate::logger::logger::warn;

use super::rwal_params::OrderBy;

pub fn get_palette(
    image: &RgbImage,
    accent_color: u32,
    clamp_range: (f32, f32),
    order: OrderBy,
) -> Vec<String> {
    let colors = get_colors(image);
    let clamped_colors = clamp_colors(&colors, clamp_range.0, clamp_range.1);
    let lab_colors = colors_to_lab(&clamped_colors);
    let clusters = get_clusters(&lab_colors);

    let mut hsv_palette = collect_hsv_palette(clusters);
    hsv_palette = add_missing_colors(hsv_palette);
    hsv_palette = order_palette(hsv_palette, order);

    let mut palette = prepare_colors(hsv_palette, accent_color);

    palette.extend(palette.clone());
    palette
}

fn collect_hsv_palette(clusters: Kmeans<Lab>) -> Vec<Hsv> {
    clusters
        .centroids
        .iter()
        .map(|lab| {
            let cs_lab = CSLab::new(lab.l as f64, lab.a as f64, lab.b as f64);
            Hsv::from(cs_lab)
        })
        .collect()
}

fn add_missing_colors(p: Vec<Hsv>) -> Vec<Hsv> {
    let mut hsv_colors = p.clone();

    while hsv_colors.len() < 6 {
        warn("Adding missing colors to palette");
        hsv_colors.push(Hsv::new(0.0, 0.0, 100.0));
    }

    hsv_colors
}

fn order_palette(p: Vec<Hsv>, order: OrderBy) -> Vec<Hsv> {
    let mut hsv_colors = p.clone();
    match order {
        OrderBy::Hue => hsv_colors.sort_by(|a, b| a.h.partial_cmp(&b.h).unwrap()),
        OrderBy::Saturation => hsv_colors.sort_by(|a, b| a.s.partial_cmp(&b.s).unwrap()),
        OrderBy::Brightness => hsv_colors.sort_by(|a, b| a.v.partial_cmp(&b.v).unwrap()),
        OrderBy::Semantic => {
            let semantic_hues = [360.0, 120.0, 60.0, 240.0, 300.0, 180.0];
            let mut sorted_palette = Vec::new();
            let mut used_indices = std::collections::HashSet::new();

            for &target_hue in &semantic_hues {
                let mut closest_index = 0;
                let mut closest_diff = f64::MAX;

                for (i, hsv) in hsv_colors.iter().enumerate() {
                    if !used_indices.contains(&i) {
                        let diff = (hsv.h - target_hue).abs();
                        if diff < closest_diff {
                            closest_diff = diff;
                            closest_index = i;
                        }
                    }
                }

                used_indices.insert(closest_index);
                sorted_palette.push(hsv_colors[closest_index].clone());
            }

            hsv_colors = sorted_palette;
        }
    }
    hsv_colors
}

fn prepare_colors(p: Vec<Hsv>, accent_color: u32) -> Vec<String> {
    let accent_index = accent_color.clamp(0, 5) as usize;
    let accent_rgb = Rgb::from(p[accent_index]);

    let bg_color = merge_rgb(Rgb::new(0.0, 0.0, 0.0), accent_rgb);
    let fg_color = merge_rgb(Rgb::new(255.0, 255.0, 255.0), accent_rgb);

    let mut result = Vec::with_capacity(8);
    result.push(bg_color);
    result.extend(p.into_iter().take(6).map(Rgb::from));
    result.push(fg_color);

    result
        .iter()
        .map(|rgb| {
            let components = [
                (rgb.r as u8).clamp(0, 255),
                (rgb.g as u8).clamp(0, 255),
                (rgb.b as u8).clamp(0, 255),
            ];
            format!(
                "#{:02X}{:02X}{:02X}",
                components[0], components[1], components[2]
            )
        })
        .collect()
}

fn get_colors(image: &RgbImage) -> Vec<Rgb> {
    image
        .pixels()
        .map(|pixel| Rgb::new(pixel[0] as f64, pixel[1] as f64, pixel[2] as f64))
        .collect()
}

fn clamp_colors(colors: &[Rgb], min_v: f32, max_v: f32) -> Vec<u8> {
    let min_clamp = min_v / 255.0;
    let max_clamp = max_v / 255.0;

    colors
        .iter()
        .flat_map(|color| {
            let mut hsv = Hsv::from_rgb(color);
            hsv.v = hsv.v.clamp(min_clamp as f64, max_clamp as f64);
            let rgb = hsv.to_rgb();
            [rgb.r as u8, rgb.g as u8, rgb.b as u8]
        })
        .collect()
}

fn colors_to_lab(colors: &[u8]) -> Vec<Lab> {
    from_component_slice::<Srgb<u8>>(colors)
        .iter()
        .map(|x| x.into_format::<f32>().into_color())
        .collect()
}

fn get_clusters(lab_colors: &[Lab]) -> Kmeans<Lab> {
    (0..3)
        .map(|i| get_kmeans(6, 100, 0.001, false, lab_colors, 64 + i as u64))
        .min_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(Ordering::Equal))
        .unwrap()
}

fn merge_rgb(a: Rgb, b: Rgb) -> Rgb {
    Rgb::new(
        (4.0 * a.r + b.r) / 5.0,
        (4.0 * a.g + b.g) / 5.0,
        (4.0 * a.b + b.b) / 5.0,
    )
}

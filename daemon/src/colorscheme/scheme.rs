use std::{path::Path, thread};

use crate::colorscheme::rwal::rwal_params::OrderBy;
use crate::daemon::config::Config;
use crate::encode_string;
use crate::expand_user;
use crate::get_image_name;
use crate::logger::logger::{err, log};
use crate::template::template::Template;
use crate::wallpaper::image::ImageOperations;
use crate::COLORS_DIR;

use super::rwal::{
    actions::{cache_rwal, run_rwal},
    rwal_params::RwalParams,
};

pub fn set_scheme(config: &Config, image_path: &str) {
    if let Some(image_ops) = &config.image_operations {
        if let Some(rwal_params) = &config.rwal_params {
            let cache_path = get_cache_path(image_ops, rwal_params, image_path);

            if !Path::new(&cache_path).exists() {
                cache_scheme(config, image_path);
            }

            let colors = run_rwal(image_path, &cache_path, rwal_params, image_ops);

            if let Some(templates) = &config.templates {
                let mut handlers = Vec::new();

                log("Applying templates...");

                for template in templates {
                    let template = template.clone();
                    let colors = colors.clone();
                    let thread = thread::spawn(move || {
                        if let Ok(tem) = Template::new(&expand_user(&template)) {
                            tem.apply(colors);
                        }
                    });
                    handlers.push(thread);
                }

                for handle in handlers {
                    handle.join().unwrap()
                }
            }
        }
    }
}

pub fn cache_scheme(config: &Config, image_path: &str) {
    if config.image_operations.is_none() {
        err("Failed to cache colorscheme.");
        err("Image operations is None.");
        return;
    }
    if config.rwal_params.is_none() {
        err("Failed to cache colorscheme.");
        err("Rwal params is None.");
        return;
    }

    let image_ops = config.image_operations.as_ref().unwrap();
    let rwal_params = config.rwal_params.as_ref().unwrap();

    cache_rwal(
        image_path,
        &get_cache_path(image_ops, rwal_params, image_path),
        rwal_params,
        image_ops,
    );
}

pub fn get_cached_colors(config: &Config, image_path: &str) -> Option<Vec<String>> {
    if config.image_operations.is_none() {
        err("Failed to cache colorscheme.");
        err("Image operations is None.");
        return None;
    }
    if config.rwal_params.is_none() {
        err("Failed to cache colorscheme.");
        err("Rwal params is None.");
        return None;
    }

    let image_ops = config.image_operations.as_ref().unwrap();
    let rwal_params = config.rwal_params.as_ref().unwrap();

    if !Path::new(&get_cache_path(image_ops, rwal_params, image_path)).exists() {
        cache_scheme(config, image_path);
    }

    Some(run_rwal(
        image_path,
        &get_cache_path(image_ops, rwal_params, image_path),
        rwal_params,
        image_ops,
    ))
}

fn get_cache_path(
    image_ops: &ImageOperations,
    rwal_params: &RwalParams,
    image_path: &str,
) -> String {
    format!(
        "{}/{}",
        expand_user(COLORS_DIR),
        encode_string(&format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}",
            get_image_name(image_path),
            image_ops.brightness,
            image_ops.contrast,
            image_ops.hue,
            image_ops.invert,
            match rwal_params.order {
                OrderBy::Hue => "H",
                OrderBy::Saturation => "S",
                OrderBy::Brightness => "V",
                OrderBy::Semantic => "sem",
            },
            rwal_params.accent_color,
            rwal_params.clamp_range.0,
            rwal_params.clamp_range.1,
            rwal_params.thumb_range.0,
            rwal_params.thumb_range.1,
            rwal_params.colors,
        ))
    )
}

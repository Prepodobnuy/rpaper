use hex;
use sha2::{Sha256, Digest};

use crate::utils::config::ImageOperations;
use crate::utils::directory::expand_user;
use crate::displays::Display;


pub struct ImageMeta {
    original_image_path: String,
    cached_images_paths: Vec<String>,
    shasum: String,
}

impl ImageMeta {
    pub fn new(
        original_image_path: &str, 
        cache_path: &str,
        displays:  Option<&Vec<Display>>,
        image_ops: Option<&ImageOperations>,
        crop_mode: Option<&str>,
    ) -> Self {

        let original_image_path: String = String::from(original_image_path);
        let image_name = get_name(&original_image_path);
        let mut cached_images_paths: Vec<String> = Vec::new();
        let shasum;

        let parts: Vec<&str> = image_name.split('.').collect();
        let extension: Option<String>;

        if let Some(last_part) = parts.last() {
            extension = Some(last_part.to_string());
        } else {
            extension = None;
        }

        if let Some(image_ops) = image_ops {
            shasum = hash(&get_img_ops_affected_name(&image_name, image_ops));
        } else {
            shasum = hash(&image_name);
        }

        if let Some(displays) = displays {
            for display in displays {
                let mut image_name = format!(
                    "{}.{}.{}.{}.{}-{}",
                    display.name, display.w, display.h, display.x, display.y, image_name
                );
                if let Some(image_ops) = image_ops {
                    image_name = get_img_ops_affected_name(&image_name, image_ops);
                }
                if let Some(crop_mode) = crop_mode {
                    image_name = format!("{}{}", crop_mode, image_name);
                }

                let _extension: &str;

                if let Some(ref extension) = extension {
                    _extension = &extension
                } else {
                    _extension = "png"
                }

                image_name = hash(&image_name);

                if cache_path.trim().ends_with('/') {
                    cached_images_paths.push(
                        expand_user(&format!("{}{}.{}", cache_path, image_name, _extension))
                    )
                } else {
                    cached_images_paths.push(
                        expand_user(&format!("{}/{}.{}", cache_path, image_name, _extension))
                    )
                }
            }
        }
        
        ImageMeta {
            original_image_path,
            cached_images_paths,
            shasum,
        }
    }

    pub fn shasum(&self) -> String {
        self.shasum.clone()
    }

    pub fn image(&self) -> String {
        get_name(&self.original_image_path)
    }

    pub fn image_path(&self) -> String {
        self.original_image_path.clone()
    }

    pub fn cache_names(&self) -> Vec<String> {
        self.cached_images_paths.clone().into_iter()
            .map(|cache_path| {
                get_name(&cache_path)
            })
            .collect()
    }

    pub fn cache_paths(&self) -> Vec<String> {
        self.cached_images_paths.clone()
    }
}

fn get_name(path: &str) -> String {
    if !path.contains("/") {
        return String::from(path);
    }

    if let Some(name) = path.split("/").last() {
        return String::from(name);
    }

    String::from(path)
}

fn hash(string: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(string);

    hex::encode(hasher.finalize())
}

pub fn get_img_ops_affected_name(image_name: &str, image_ops: &ImageOperations) -> String {
    let mut image_name: String = String::from(image_name);

    if image_ops.change_contrast {
        image_name = format!("CR{}{}", image_ops.contrast, image_name)
    }
    if image_ops.change_brightness {
        image_name = format!("BR{}{}", image_ops.brightness, image_name)
    }
    if image_ops.change_huerotate {
        image_name = format!("HUE{}{}", image_ops.huerotate, image_name)
    }
    if image_ops.change_blur {
        image_name = format!("BLUR{}{}", image_ops.blur, image_name)
    }
    if image_ops.flip_h {
        image_name = format!("H_FL{}", image_name)
    }
    if image_ops.flip_v {
        image_name = format!("V_FL{}", image_name)
    }
    if image_ops.invert {
        image_name = format!("INV{}", image_name)
    }

    image_name
}
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, thread};

use crate::colorscheme::rwal::rwal_params::{OrderBy, RwalParams};
use crate::colorscheme::scheme::{cache_scheme, get_cached_colors, set_scheme};
use crate::logger::logger::log;
use crate::wallpaper::display::{
    cache_wallpaper, get_cached_image_names, get_cached_image_paths, set_wallpaper,
};
use crate::wallpaper::image::ImageOperations;
use crate::{expand_user, unix_timestamp, WALLPAPERS_DIR};
use common::Request;
use rand::rng;
use rand::seq::IndexedRandom;
use serde_json::{json, Map, Value};

use super::config::Config;

#[derive(Clone)]
pub struct RequestHandler {
    config: Config,
    message: String,
}

impl RequestHandler {
    pub fn new(config: Config, message: String) -> Self {
        if let Some(call_file) = &config.last_call_file {
            let _ = fs::write(call_file, &message);
        }
        RequestHandler { config, message }
    }

    pub fn handle_image_request(&mut self, request: &Request, respond: &mut Value) -> Result<String, String> {
        let image_path = expand_user(&request.image.clone().unwrap());

        if !Path::new(&image_path).exists() {
            return Err("path do not exists".to_string());
        }

        if request.get_c_cache {
            add_key_to_value(
                respond,
                "c_cache",
                if let Some(colors) = get_cached_colors(&self.config, &image_path) {
                    Value::Array(
                        colors
                            .iter()
                            .map(|c| Value::String(c.clone()))
                            .collect::<Vec<Value>>(),
                    )
                } else {
                    Value::Null
                },
            )
        }

        if request.get_w_cache {
            add_key_to_value(
                respond,
                "w_cache",
                if let Some(displays) = &self.config.displays {
                    if let Some(img_ops) = &self.config.image_operations {
                        get_cached_image_paths(
                            &get_cached_image_names(&displays, &img_ops, &image_path),
                            &expand_user(WALLPAPERS_DIR),
                        )
                        .iter()
                        .map(|el| Value::String(el.clone()))
                        .collect()
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Null
                },
            );
        }

        let config = collect_config_from_request(self.config.clone(), &request);

        if is_dir(&image_path) {
            log(&format!("Collecting all wallpapers from {}.", &image_path));
            let wallpapers = get_images_from_dir(&image_path);
            let mut processed_images: u128 = 0;
            if request.affect_all {
                log(&format!(
                    "Applying request for all images from {}.",
                    &image_path
                ));

                for chunk in wallpapers.chunks(4) {
                    let mut handlers = Vec::new();

                    for wallpaper in chunk {
                        processed_images += 1;
                        let wallpaper = wallpaper.clone();
                        let config = config.clone();
                        let request = request.clone();
                        let thread = thread::spawn(move || {
                            process_request(&request, &config, &wallpaper);
                        });
                        handlers.push(thread);
                    }

                    for handle in handlers {
                        handle.join().unwrap();
                    }
                }
            } else {
                let wallpaper = select_random(wallpapers);
                process_request(&request, &config, &wallpaper);
                processed_images = 1;
            }
            let end_time = unix_timestamp();

            add_key_to_value(respond, "end_time", json!(end_time));
            return Ok(format!("processed {} images", processed_images));
        }

        if !is_file_image(&image_path) {
            return Err("file is not an image or has unsuported format".to_string());
        }

        process_request(&request, &config, &image_path);
        Ok("request processed".to_string())
    }

    pub fn handle(&mut self) -> String {
        let start_time = unix_timestamp();

        let request_result = serde_json::from_str::<Request>(&self.message);

        let mut respond = Value::Object(Map::new());
        add_key_to_value(&mut respond, "start_time", json!(start_time));

        if request_result.is_err() {
            let end_time = unix_timestamp();
            let time_elapsed = end_time - start_time;

            log(&format!("Request processed in {}ms.", time_elapsed,));

            add_key_to_value(&mut respond, "end_time", json!(end_time));
            add_key_to_value(&mut respond, "time_elapsed", json!(time_elapsed));

            add_key_to_value(
                &mut respond,
                "error",
                json!("error while deserializing request"),
            );
            if let Ok(msg) = serde_json::to_string(&respond) {
                return msg;
            }
            return "{error: \"caught unexpected error while serializing responce\"}"
                .to_string();
        }

        let request = request_result.unwrap();

        // handle requests which does not require image
        if request.get_config {
            if let Ok(value) = serde_json::to_string(&self.config.clone()) {
                add_key_to_value(&mut respond, "config", Value::String(value));
            }
        }
        if request.get_current_colorscheme {}

        // handle requests which does require image
        if request.image.is_some() {
            match self.handle_image_request(&request, &mut respond) {
                Ok(msg) => {
                    add_key_to_value(&mut respond, "message", Value::String(msg));
                },
                Err(msg) => {
                    add_key_to_value(&mut respond, "error", Value::String(msg));
                },
            }
        }
        
        let end_time = unix_timestamp();
        let time_elapsed = end_time - start_time;

        log(&format!("Request processed in {}ms.", time_elapsed,));

        add_key_to_value(&mut respond, "end_time", json!(end_time));
        add_key_to_value(&mut respond, "time_elapsed", json!(time_elapsed));
        if let Ok(msg) = serde_json::to_string(&respond) {
            return msg;
        }
        return "{message: \"caught unexpected error while serializing responce\"}".to_string();
    }
}

fn collect_config_from_request(mut config: Config, request: &Request) -> Config {
    if let Some(displays) = &request.displays {
        config.displays = Some(displays.iter().map(|d| d.clone()).collect())
    }
    if let Some(templates) = &request.templates {
        config.templates = Some(templates.iter().map(|t| t.clone()).collect())
    }
    if let Some(set_command) = &request.set_command {
        config.set_command = Some(set_command.clone())
    }
    if let Some(resize_alg) = &request.resize_alg {
        config.resize_algorithm = Some(resize_alg.clone())
    }

    let mut rwal_params: Option<RwalParams> = None;

    if let Some(_rwal_params) = config.rwal_params {
        let mut thumb_range = _rwal_params.thumb_range;
        let mut clamp_range = _rwal_params.clamp_range;
        let mut accent_color = _rwal_params.accent_color;
        let colors = _rwal_params.colors;
        let mut order = _rwal_params.order;

        if let Some(thumb) = &request.rwal_thumb {
            if let Ok(value) = get_range_from_str::<u32>(&thumb) {
                thumb_range = value;
            }
        }
        if let Some(clamp) = &request.rwal_clamp {
            if let Ok(value) = get_range_from_str::<f32>(&clamp) {
                clamp_range = value;
            }
        }
        if let Some(color) = request.rwal_accent {
            accent_color = color
        }
        if let Some(_order) = &request.rwal_order {
            order = match OrderBy::from_str(&_order) {
                Ok(value) => value,
                Err(_) => OrderBy::Hue,
            }
        }

        rwal_params = Some(RwalParams::new(
            thumb_range,
            clamp_range,
            accent_color,
            colors,
            order,
        ));
    }

    config.rwal_params = rwal_params;

    let mut image_ops: Option<ImageOperations> = None;

    if let Some(_image_ops) = config.image_operations {
        let mut contrast = _image_ops.contrast;
        let mut brightness = _image_ops.brightness;
        let mut hue = _image_ops.hue;
        let mut blur = _image_ops.blur;
        let mut invert = _image_ops.invert;
        let mut flip_h = _image_ops.flip_h;
        let mut flip_v = _image_ops.flip_v;

        if let Some(_contrast) = request.contrast {
            contrast = _contrast
        }
        if let Some(_brightness) = request.brightness {
            brightness = _brightness
        }
        if let Some(_hue) = request.hue {
            hue = _hue
        }
        if let Some(_blur) = request.blur {
            blur = _blur
        }
        if let Some(_invert) = request.invert {
            invert = _invert
        }
        if let Some(_flip_h) = request.flip_h {
            flip_h = _flip_h
        }
        if let Some(_flip_v) = request.flip_v {
            flip_v = _flip_v
        }

        image_ops = Some(ImageOperations::new(
            contrast, brightness, hue, blur, invert, flip_h, flip_v,
        ));
    }

    config.image_operations = image_ops;

    config
}

fn process_request(request: &Request, config: &Config, image_path: &str) {
    log(&format!("Processing image {}", &image_path,));
    if request.c_cache && !request.c_set {
        log(&format!("Caching colors for {}", &image_path,));
        cache_scheme(config, image_path);
    }
    if request.w_cache && !request.w_set {
        log(&format!("Caching wallpapers for {}", &image_path,));
        cache_wallpaper(config, image_path);
    }
    if request.c_set {
        log(&format!("Setting colors for {}", &image_path,));
        set_scheme(config, image_path);
    }
    if request.w_set {
        log(&format!("Setting wallpapers for {}", &image_path,));
        set_wallpaper(config, image_path);
    }
}

fn get_range_from_str<T: std::str::FromStr>(s: &str) -> Result<(T, T), ()> {
    let values = s.split("X").collect::<Vec<&str>>();
    if values.len() != 2 {
        return Err(());
    }

    let first = values[0].parse::<T>().map_err(|_| ())?;
    let second = values[1].parse::<T>().map_err(|_| ())?;

    Ok((first, second))
}

fn is_dir(path: &str) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
}

fn select_random(strings: Vec<String>) -> String {
    let mut rng = rng();

    if let Some(random_string) = strings.choose(&mut rng) {
        random_string.to_string()
    } else {
        panic!("Directory is empty")
    }
}

fn is_file_image(path: &str) -> bool {
    if let Some(extension) = path.split(".").last() {
        return matches!(
            extension.to_lowercase().as_str(),
            "jpg" | "jpeg" | "webp" | "png" | "gif" | "bmp" | "tiff"
        );
    }
    false
}

fn get_absolute_path(path: String) -> String {
    let Ok(path) = PathBuf::from_str(&path);

    return path
        .canonicalize()
        .unwrap_or_else(|_| path)
        .to_string_lossy()
        .to_string();
}

fn get_images_from_dir(dir: &str) -> Vec<String> {
    let path = Path::new(dir);
    let files = fs::read_dir(path).unwrap();
    let mut res: Vec<String> = Vec::new();

    for entry in files {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            res.extend(get_images_from_dir(&get_absolute_path(
                entry.path().to_string_lossy().to_string(),
            )))
        } else if file_type.is_file() {
            if let Some(extension) = entry.path().extension() {
                if is_file_image(extension.to_str().unwrap_or("")) {
                    res.push(get_absolute_path(
                        entry.path().to_string_lossy().to_string(),
                    ))
                }
            }
        }
    }
    res
}

fn add_key_to_value(value: &mut Value, key: &str, new_value: Value) {
    if let Value::Object(ref mut map) = value {
        map.insert(key.to_string(), new_value);
    }
}

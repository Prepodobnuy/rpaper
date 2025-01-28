use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
use std::{fs, thread};

use crate::colorscheme::rwal::{OrderBy, RwalParams};
use crate::colorscheme::scheme::{cache_scheme, get_cached_colors, set_scheme};
use crate::logger::logger::log;
use crate::{expand_user, unix_timestamp, COLORS_PATH, WALLPAPERS_DIR};
use crate::wallpaper::display::{cache_wallpaper, get_cached_image_names, get_cached_image_paths, set_wallpaper, Display, ImageOperations, WCacheInfo};
use crate::colorscheme::template::Template;

use super::config::{Config, JsonString};
use super::daemon::{InfoRequest, MpscData};


pub struct Request {
    config: Config,
    message: String,
}



impl Request {
    pub fn new(config: Config, message: String) -> Self {
        Request {
            config,
            message,
        }
    }
    pub fn process(&mut self) {
        let current_timestamp = unix_timestamp();
        let messages: Vec<String> = self.message.split(";")
            .map(|s| s.to_string())
            .collect();

        for chunk in messages.chunks(2) {
            let mut handlers = Vec::new(); 
            for message in chunk {
                let message = message.clone();
                let mut config = self.config.clone();
    
                let thread = thread::spawn(move || {
                    let request_tags = RequestTags::new(&message);
    
                    if let Some(displays) = request_tags.config_displays {
                        config.displays = Some(displays)
                    }
                    if let Some(templates) = request_tags.config_templates {
                        config.templates = Some(templates)
                    }
                    if let Some(wallpaper_set_command) = request_tags.set_command {
                        config.wallpaper_set_command = Some(wallpaper_set_command)
                    }
                    if let Some(resize_alg) = request_tags.config_resize_alg {
                        config.resize_algorithm = Some(resize_alg)
                    }
    
                    let mut rwal_params: Option<RwalParams> = None;
    
                    if let Some(_rwal_params) = config.rwal_params {
                        let mut thumb_range = _rwal_params.thumb_range;
                        let mut clamp_range = _rwal_params.clamp_range;
                        let mut accent_color = _rwal_params.accent_color;
                        let mut colors = _rwal_params.colors;
                        let mut order = _rwal_params.order;
    
                        if let Some(thumb) = request_tags.rwal_thumb {
                            thumb_range = thumb
                        }
                        if let Some(clamp) = request_tags.rwal_clamp {
                            clamp_range = clamp
                        }
                        if let Some(color) = request_tags.rwal_accent {
                            accent_color = color
                        }
                        if let Some(cols) = request_tags.rwal_count {
                            colors = cols
                        }
                        if let Some(_order) = request_tags.rwal_order {
                            order = _order
                        }
    
                        rwal_params = Some(RwalParams::new(thumb_range, clamp_range, accent_color, colors, order));
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
    
                        if let Some(_contrast) = request_tags.config_contrast {
                            contrast = _contrast
                        }
                        if let Some(_brightness) = request_tags.config_brightness {
                            brightness = _brightness
                        }
                        if let Some(_hue) = request_tags.config_hue {
                            hue = _hue
                        }
                        if let Some(_blur) = request_tags.config_blur {
                            blur = _blur
                        }
                        if let Some(_invert) = request_tags.config_invert {
                            invert = _invert
                        }
                        if let Some(_flip_h) = request_tags.config_flip_h {
                            flip_h = _flip_h
                        }
                        if let Some(_flip_v) = request_tags.config_flip_v {
                            flip_v = _flip_v
                        }
    
                        image_ops = Some(ImageOperations::new(contrast, brightness, hue, blur, invert, flip_h, flip_v));
                    }
    
                    config.image_operations = image_ops;
    
                    if let Some(image) = request_tags.image {
                        let mut _handlers = Vec::new();
                        
                        if let Some(w_cache) = request_tags.w_cache {
                            if w_cache {
                                let config = config.clone();
                                let image_path = image.clone();
                                let _thread = thread::spawn(move || {
                                    cache_wallpaper(&config, &image_path);
                                });
                                _handlers.push(_thread);
                            }
                        }
                        
                        if let Some(w_set) = request_tags.w_set {
                            if w_set {
                                let config = config.clone();
                                let image_path = image.clone();
                                let _thread = thread::spawn(move || {
                                    set_wallpaper(&config, &image_path);
                                });
                                _handlers.push(_thread);
                            }
                        }
                        
                        if let Some(c_cache) = request_tags.c_cache {
                            if c_cache {
                                let config = config.clone();
                                let image_path = image.clone();
                                let _thread = thread::spawn(move || {
                                    cache_scheme(&config, &image_path);
                                });
                                _handlers.push(_thread);
                            }
                        }
                        
                        if let Some(c_set) = request_tags.c_set {
                            if c_set {
                                let config = config.clone();
                                let image_path = image.clone();
                                let _thread = thread::spawn(move || {
                                    set_scheme(&config, &image_path);
                                });
                                _handlers.push(_thread);
                            }
                        }
                        
                        for handler in _handlers {
                            handler.join().unwrap();
                        }
                    }
    
                });
    
                handlers.push(thread)
            }
            for handler in handlers {
                handler.join().unwrap();
            }
        }
        
        log(&format!("Request processed in {}ms.", unix_timestamp() - current_timestamp));
    }
}

struct RequestTags {
    // Main tags
    w_set: Option<bool>,
    w_cache: Option<bool>,
    c_set: Option<bool>,
    c_cache: Option<bool>,
    image: Option<String>,
    set_command: Option<String>,
    // Config tags
    config_contrast: Option<f32>,
    config_brightness: Option<i32>,
    config_hue: Option<i32>,
    config_blur: Option<f32>,
    config_invert: Option<bool>,
    config_flip_h: Option<bool>,
    config_flip_v: Option<bool>,
    config_displays: Option<Vec<Display>>,
    config_templates: Option<Vec<Template>>,
    config_resize_alg: Option<String>,
    // Rwal tags
    rwal_thumb: Option<(u32, u32)>,
    rwal_clamp: Option<(f32, f32)>,
    rwal_count: Option<u32>,
    rwal_accent: Option<u32>,
    rwal_order: Option<OrderBy>,
}

impl RequestTags {
    fn new(message: &str) -> Self {
        let mut w_set: Option<bool> = None;
        let mut w_cache: Option<bool> = None;
        let mut c_set: Option<bool> = None;
        let mut c_cache: Option<bool> = None;
        let mut image: Option<String> = None;
        let mut set_command: Option<String> = None;
        let mut config_contrast: Option<f32> = None;
        let mut config_brightness: Option<i32> = None;
        let mut config_hue: Option<i32> = None;
        let mut config_blur: Option<f32> = None;
        let mut config_invert: Option<bool> = None;
        let mut config_flip_h: Option<bool> = None;
        let mut config_flip_v: Option<bool> = None;
        let mut config_displays: Option<Vec<Display>> = None;
        let mut config_templates: Option<Vec<Template>> = None;
        let mut config_resize_alg: Option<String> = None;
        let mut rwal_thumb: Option<(u32, u32)> = None;
        let mut rwal_clamp: Option<(f32, f32)> = None;
        let mut rwal_count: Option<u32> = None;
        let mut rwal_accent: Option<u32> = None;
        let mut rwal_order: Option<OrderBy> = None;

        let tags: Vec<String> = message.split("    ").map(|s| s.to_string()).collect();

        for (i, tag) in tags.clone().into_iter().enumerate() {
            match tag.as_str() {
                "W_SET" =>             {w_set             = Some(true)},
                "W_CACHE" =>           {w_cache           = Some(true)}, 
                "C_SET" =>             {c_set             = Some(true)},
                "C_CACHE" =>           {c_cache           = Some(true)},
                "IMAGE" =>             {image             = get_value::<String>(&tags, i)},
                "CONFIG_CONTRAST" =>   {config_contrast   = get_value::<f32>(&tags, i)},
                "CONFIG_BRIGHTNESS" => {config_brightness = get_value::<i32>(&tags, i)},
                "CONFIG_HUE" =>        {config_hue        = get_value::<i32>(&tags, i)},
                "CONFIG_BLUR" =>       {config_blur       = get_value::<f32>(&tags, i)},
                "CONFIG_INVERT" =>     {config_invert     = Some(true)},
                "CONFIG_FLIP_H" =>     {config_flip_h     = Some(true)},
                "CONFIG_FLIP_V" =>     {config_flip_v     = Some(true)},
                "CONFIG_DISPLAYS" =>   {config_displays   = get_array::<Display>(&tags, i)},
                "CONFIG_TEMPLATES" =>  {config_templates  = get_array::<Template>(&tags, i)},
                "CONFIG_RESIZE_ALG" => {config_resize_alg = get_value::<String>(&tags, i)},
                "RWAL_THUMB" =>        {rwal_thumb        = get_thumb(&tags, i)},
                "RWAL_CLAMP" =>        {rwal_clamp        = get_clamp(&tags, i)},
                "RWAL_COUNT" =>        {rwal_count        = get_value::<u32>(&tags, i)},
                "RWAL_ACCENT" =>       {rwal_accent       = get_value::<u32>(&tags, i)},
                "RWAL_ORDER" =>        {rwal_order        = get_rwal_order(&tags, i)},
                "SET_COMMAND" =>       {set_command       = get_value::<String>(&tags, i)},
                _ => {},
            }
        }

        RequestTags {
            w_set,
            w_cache,
            c_set,
            c_cache,
            image,
            set_command,
            config_contrast,
            config_brightness,
            config_hue,
            config_blur,
            config_invert,
            config_flip_h,
            config_flip_v,
            config_displays,
            config_templates,
            config_resize_alg,
            rwal_thumb,
            rwal_clamp,
            rwal_count,
            rwal_accent,
            rwal_order,
        }
    }
}

fn get_value<T: std::str::FromStr>(tags: &Vec<String>, index: usize) -> Option<T> {
    if index + 1 < tags.len() {
        if let Ok(value) = tags[index + 1].parse::<T>() {
            return Some(value);
        }
    }
    None
}

fn get_array<T: std::str::FromStr>(tags: &Vec<String>, index: usize) -> Option<Vec<T>> {
    if index + 1 < tags.len() {
        if let Some(raw_displays) = get_value::<String>(tags, index) {
            let mut displays: Vec<T> = Vec::new();
            raw_displays.split(",")
                .for_each(|raw_display| {
                    if let Ok(display) = T::from_str(&raw_display) {
                        displays.push(display)
                    }
                });
            return Some(displays);
        }
    }
    None
}

fn get_clamp(tags: &Vec<String>, index: usize) -> Option<(f32, f32)> {
    if index + 1 < tags.len() {
        if let Some(raw_clamp) = get_value::<String>(tags, index) {
            let clamp: Vec<String> = raw_clamp.split("X")
                .map(|s| s.to_string())
                .collect();
            if clamp.len() == 2 {
                if let Ok(min) = clamp[0].parse::<f32>() {
                    if let Ok(max) = clamp[1].parse::<f32>() {
                        return Some((min, max));
                    }
                }
            }
        }
    }
    None
}

fn get_thumb(tags: &Vec<String>, index: usize) -> Option<(u32, u32)> {
    if index + 1 < tags.len() {
        if let Some(raw_thumb) = get_value::<String>(tags, index) {
            let thumb: Vec<String> = raw_thumb.split("X")
                .map(|s| s.to_string())
                .collect();
            if thumb.len() == 2 {
                if let Ok(min) = thumb[0].parse::<u32>() {
                    if let Ok(max) = thumb[1].parse::<u32>() {
                        return Some((min, max));
                    }
                }
            }
        }
    }
    None
}

fn get_rwal_order(tags: &Vec<String>, index: usize) -> Option<OrderBy> {
    if index + 1 < tags.len() {
        if let Some(order) = get_value::<String>(tags, index) {
            match order.as_str() {
                "h" | "H" => {return Some(OrderBy::Hue);},
                "s" | "S" => {return Some(OrderBy::Saturation);},
                "v" | "V" | "b" | "B" => {return Some(OrderBy::Brightness);},
                _ => {},
            }
        } 
    }
    None
}

pub fn process_info_request (config: Config, request: InfoRequest) -> Option<String> {
    match request {
        InfoRequest::DisplaysRequest => {
            log("Displays request received.");
            if let Some(displays) = config.displays {
                Some(displays.json())
            } else {
                None
            }
        },
        InfoRequest::TemplatesRequest => {
            log("Templates request received.");
            if let Some(templates) = config.templates {
                Some(templates.json())
            } else {
                None
            }
        },
        InfoRequest::CurrentColorSchemeRequest => {
            log("Current colorscheme request received.");
            if !Path::new(&expand_user(COLORS_PATH)).exists() {
                return None;
            }
            if let Ok(data) = fs::read_to_string(&expand_user(COLORS_PATH)) {
                return Some(format!(
                    "[{}]",
                    data.split("\n")
                        .into_iter()
                        .map(|x| {
                            format!("\"{x}\"")
                        })
                        .collect::<Vec<String>>()
                        .join(",")
                ));
            }
            None
        }
        InfoRequest::ImageOpsRequest => {
            log("Image operations request received.");
            if let Some(image_ops) = config.image_operations {
                Some(image_ops.json())
            } else {
                None
            }
        },
        InfoRequest::RwalParamsRequest => {
            log("Rwal params request received.");
            if let Some(rwal_params) = config.rwal_params {
                Some(rwal_params.json())
            } else {
                None
            }
        },
        InfoRequest::ConfigRequest => {
            log("Config request received.");
            Some(config.json())
        },
        InfoRequest::WallpaperCacheRequest(val) => {
            log("Image cache info request received.");
            if let Some(img_ops) = &config.image_operations {
                if let Some(displays) = &config.displays {
                    let cached_image_paths = get_cached_image_paths(
                        &get_cached_image_names(&displays, &img_ops, &val),
                        WALLPAPERS_DIR,
                    );

                    for cache_path in &cached_image_paths {
                        if !Path::new(&expand_user(cache_path)).exists() {
                            cache_wallpaper(&config, &val);
                            break;
                        }
                    }

                    let mut w_caches = Vec::new(); 
                    for (i, path) in cached_image_paths.into_iter().enumerate() {
                        w_caches.push(WCacheInfo::new(&displays[i].name(), &path));
                    }

                    Some(format!(
                        "[{}]",
                        w_caches.into_iter()
                            .map(|x| {
                                x.json()
                            })
                            .collect::<Vec<String>>()
                            .join(",")
                    ))

                }
                else {None}
            }
            else {None}
        },
        InfoRequest::ColoschemeCacheRequest(val) => {
            log("Colorscheme cache info request received.");
            if let Some(colors) = get_cached_colors(&config, &val) {
                return Some(format!(
                    "[{}]",
                    colors.into_iter()
                        .map(|x| {
                            format!("\"{x}\"")
                        })
                        .collect::<Vec<String>>()
                        .join(",")
                ));
            }
            None
        }
        _ => {None}
    }
}

pub fn handle_request(request: &str, tx: &mpsc::Sender<MpscData>, listener_rx: &Receiver<MpscData>) -> Option<String> {
    let info_patterns = [
        "GET_DISPLAYS",
        "GET_TEMPLATES",
        "GET_SCHEME",
        "GET_IMAGE_OPS",
        "GET_RWAL_PARAMS",
        "GET_CONFIG",
        "GET_W_CACHE",
        "GET_C_CACHE",
    ];

    for pat in info_patterns {
        if request.contains(pat) {
            let request = match pat {
                "GET_DISPLAYS"    => {MpscData::InfoRequest(InfoRequest::DisplaysRequest)},
                "GET_TEMPLATES"   => {MpscData::InfoRequest(InfoRequest::TemplatesRequest)},
                "GET_SCHEME"      => {MpscData::InfoRequest(InfoRequest::CurrentColorSchemeRequest)}
                "GET_IMAGE_OPS"   => {MpscData::InfoRequest(InfoRequest::ImageOpsRequest)},
                "GET_RWAL_PARAMS" => {MpscData::InfoRequest(InfoRequest::RwalParamsRequest)},
                "GET_CONFIG"      => {MpscData::InfoRequest(InfoRequest::ConfigRequest)},
                "GET_W_CACHE"     => {
                    if let Some(val) = get_image(request.split_whitespace().collect()) {
                        MpscData::InfoRequest(InfoRequest::WallpaperCacheRequest(val))
                    } else {
                        MpscData::InfoRequest(InfoRequest::EmptyRequest)
                    }
                },
                "GET_C_CACHE"     => {
                    if let Some(val) = get_image(request.split_whitespace().collect()) {
                        MpscData::InfoRequest(InfoRequest::ColoschemeCacheRequest(val))
                    } else {
                        MpscData::InfoRequest(InfoRequest::EmptyRequest)
                    }
                },
                _ => {MpscData::InfoRequest(InfoRequest::EmptyRequest)},
            };

            if tx.send(request.clone()).is_ok() {
                if let Ok(value) = listener_rx.recv_timeout(Duration::from_millis(10000)) {
                    if let MpscData::ListenerRespond(value) = value {
                        return Some(value);
                    }
                }
            }
        }
    }

    let _ = tx.send(MpscData::ListenerRequest(request.to_string()));

    None
}

fn get_image(parts: Vec<&str>) -> Option<String> {
    if let Some(index) = parts.iter().position(|&s| s == "IMAGE") {
        if index + 1 < parts.len() {
            return None;
        }
        return Some(parts[index + 1].to_string());
    }
    None
}
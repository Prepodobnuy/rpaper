use std::thread;

use crate::colorscheme::rwal::RwalParams;
use crate::colorscheme::scheme::{cache_scheme, set_scheme};
use crate::wallpaper::display::{cache_wallpaper, set_wallpaper, Display, ImageOperations};
use crate::colorscheme::template::Template;

use super::config::Config;


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
    
                        rwal_params = Some(RwalParams::new(thumb_range, clamp_range, accent_color, colors));
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
        
        let tags: Vec<String> = message.split("    ").map(|s| s.to_string()).collect();

        for (i, tag) in tags.clone().into_iter().enumerate() {
            match tag.as_str() {
                "W_SET" =>             {w_set             = Some(true)},
                "W_CACHE" =>           {w_cache           = Some(true)}, 
                "C_SET" =>             {c_set             = Some(true)},
                "C_CACHE" =>           {c_cache           = Some(true)},
                "IMAGE" =>             {image             = get_string(&tags, i)},
                "CONFIG_CONTRAST" =>   {config_contrast   = get_f(&tags, i)},
                "CONFIG_BRIGHTNESS" => {config_brightness = get_i(&tags, i)},
                "CONFIG_HUE" =>        {config_hue        = get_i(&tags, i)},
                "CONFIG_BLUR" =>       {config_blur       = get_f(&tags, i)},
                "CONFIG_INVERT" =>     {config_invert     = Some(true)},
                "CONFIG_FLIP_H" =>     {config_flip_h     = Some(true)},
                "CONFIG_FLIP_V" =>     {config_flip_v     = Some(true)},
                "CONFIG_DISPLAYS" =>   {config_displays   = get_displays(&tags, i)},
                "CONFIG_TEMPLATES" =>  {config_templates  = get_templates(&tags, i)},
                "CONFIG_RESIZE_ALG" => {config_resize_alg = get_string(&tags, i)},
                "RWAL_THUMB" =>        {rwal_thumb        = get_thumb(&tags, i)},
                "RWAL_CLAMP" =>        {rwal_clamp        = get_clamp(&tags, i)},
                "RWAL_COUNT" =>        {rwal_count        = get_u(&tags, i)},
                "RWAL_ACCENT" =>       {rwal_accent       = get_u(&tags, i)},
                "SET_COMMAND" =>       {set_command       = get_string(&tags, i)},
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
        }
    }
}

fn get_string(tags: &Vec<String>, index: usize) -> Option<String> {
    if index + 1 < tags.len() {
        return Some(tags[index + 1].clone());
    }
    None
}

fn get_f(tags: &Vec<String>, index: usize) -> Option<f32> {
    if index + 1 < tags.len() {
        if let Ok(value) = tags[index + 1].parse::<f32>() {
            return Some(value);
        }
    }
    None
}

fn get_i(tags: &Vec<String>, index: usize) -> Option<i32> {
    if index + 1 < tags.len() {
        if let Ok(value) = tags[index + 1].parse::<i32>() {
            return Some(value);
        }
    }
    None
}

fn get_u(tags: &Vec<String>, index: usize) -> Option<u32> {
    if index + 1 < tags.len() {
        if let Ok(value) = tags[index + 1].parse::<u32>() {
            return Some(value);
        }
    }
    None
}

fn get_displays(tags: &Vec<String>, index: usize) -> Option<Vec<Display>> {
    if index + 1 < tags.len() {
        if let Some(raw_displays) = get_string(tags, index) {
            let mut displays: Vec<Display> = Vec::new();
            raw_displays.split(",")
                .for_each(|raw_display| {
                    if let Ok(display) = Display::from_string(&raw_display) {
                        displays.push(display)
                    }
                });
            return Some(displays);
        }
    }
    None
}

fn get_templates(tags: &Vec<String>, index: usize) -> Option<Vec<Template>> {
    if index + 1 < tags.len() {
        if let Some(raw_templates) = get_string(tags, index) {
            let mut templates: Vec<Template> = Vec::new();
            raw_templates.split(",")
                .for_each(|raw_template| {
                    templates.push(Template::new(raw_template))
                });
            return Some(templates);
        }
    }
    None
}

fn get_clamp(tags: &Vec<String>, index: usize) -> Option<(f32, f32)> {
    if index + 1 < tags.len() {
        if let Some(raw_clamp) = get_string(tags, index) {
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
        if let Some(raw_thumb) = get_string(tags, index) {
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

use std::env;

#[derive(Clone)]
pub struct Args {
    pub rpaper_temp_path: Option<String>,
    pub rpaper_vars_path: Option<String>,
    pub rpaper_cache_dir: Option<String>,
    pub rpaper_scheme_file: Option<String>,
    pub rpaper_wall_command: Option<String>,
    pub rpaper_resize_algorithm: Option<String>,
    pub rpaper_cache_scheme: Option<bool>,
    pub rpaper_cache_walls: Option<bool>,
    pub rpaper_set_templates: Option<bool>,
    pub rpaper_set_walls: Option<bool>,

    pub image_processing_change_contrast: Option<bool>,
    pub image_processing_change_brigtness: Option<bool>,
    pub image_processing_change_hue: Option<bool>,
    pub image_processing_change_blur: Option<bool>,
    pub image_processing_invert: Option<bool>,
    pub image_processing_h_flip: Option<bool>,
    pub image_processing_v_flip: Option<bool>,
    pub image_processing_contrast: Option<f32>,
    pub image_processing_brigtness: Option<f32>,
    pub image_processing_hue: Option<i32>,
    pub image_processing_blur: Option<f32>,

    pub rwal_cache_dir: Option<String>,
    pub rwal_thumb_w: Option<u32>,
    pub rwal_thumb_h: Option<u32>,
    pub rwal_accent: Option<u32>,
    pub rwal_clamp_min: Option<f32>,
    pub rwal_clamp_max: Option<f32>,

    pub displays: Option<String>,
    pub templates: Option<String>, // TODO implement a one-line variation of writing templates
    pub variables: Option<String>, // TODO implement a one-line variation of writing variables
}

impl Args {
    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();

        let mut rpaper_temp_path = None;
        let mut rpaper_vars_path = None;
        let mut rpaper_cache_dir = None;
        let mut rpaper_scheme_file = None;
        let mut rpaper_wall_command = None;
        let mut rpaper_resize_algorithm = None;
        let mut rpaper_cache_scheme = None;
        let mut rpaper_cache_walls = None;
        let mut rpaper_set_templates = None;
        let mut rpaper_set_walls = None;

        let mut image_processing_change_contrast = None;
        let mut image_processing_change_brigtness = None;
        let mut image_processing_change_hue = None;
        let mut image_processing_change_blur = None;
        let mut image_processing_invert = None;
        let mut image_processing_h_flip = None;
        let mut image_processing_v_flip = None;
        let mut image_processing_contrast = None;
        let mut image_processing_brigtness = None;
        let mut image_processing_hue = None;
        let mut image_processing_blur = None;
        
        let mut rwal_cache_dir = None;
        let mut rwal_thumb_w = None;
        let mut rwal_thumb_h = None;
        let mut rwal_accent = None;
        let mut rwal_clamp_min = None;
        let mut rwal_clamp_max = None;
        
        let mut displays = None;
        let mut templates = None;
        let mut variables = None;

        for (i, arg) in args.clone().into_iter().enumerate() {
            let value = Some(args[i+1].clone());
            match arg.as_str() {
                "--temp-path" =>          rpaper_temp_path = get_string(value),
                "--vars-path" =>          rpaper_vars_path = get_string(value),
                "--cache-dir" =>          rpaper_cache_dir = get_string(value),
                "--scheme-file" =>        rpaper_scheme_file = get_string(value),
                "--wall-command" =>       rpaper_wall_command = get_string(value),
                "--resize-algorithm" =>   rpaper_resize_algorithm = get_string(value),
                "--cache-color-scheme" => rpaper_cache_scheme = get_bool(value),
                "--cache_wallpaper" =>    rpaper_cache_walls = get_bool(value),
                "--set-templates" =>      rpaper_set_templates = get_bool(value),
                "--set_wallpaper" =>      rpaper_set_walls = get_bool(value),
                
                "--change-contrast" =>  image_processing_change_contrast = get_bool(value),
                "--change-brigtness" => image_processing_change_brigtness = get_bool(value),
                "--change-hue" =>       image_processing_change_hue = get_bool(value),
                "--change-blur" =>      image_processing_change_blur = get_bool(value),
                "--invert" =>           image_processing_invert = get_bool(value),
                "--h-flip" =>           image_processing_h_flip = get_bool(value),
                "--v-flip" =>           image_processing_v_flip = get_bool(value),
                "--contrast" =>         image_processing_contrast = get_f32(value),
                "--brigtness" =>        image_processing_brigtness = get_f32(value),
                "--hue" =>              image_processing_hue = get_i32(value),
                "--blur" =>             image_processing_blur = get_f32(value),

                "--r-cache-dir" => rwal_cache_dir = get_string(value),
                "thumb_w" =>       rwal_thumb_w = get_u32(value),
                "thumb_h" =>       rwal_thumb_h = get_u32(value),
                "accent" =>        rwal_accent = get_u32(value),
                "clamp_min" =>     rwal_clamp_min = get_f32(value),
                "clamp_max" =>     rwal_clamp_max = get_f32(value),
                
                "--displays" | "-d" =>  displays = get_string(value),
                "--templates" | "-t" => templates = get_string(value),
                "--variables" | "-v" => variables = get_string(value),
                _ => {},
            }
        }

        Args {
            rpaper_temp_path,
            rpaper_vars_path,
            rpaper_cache_dir,
            rpaper_scheme_file,
            rpaper_wall_command,
            rpaper_resize_algorithm,
            rpaper_cache_scheme,
            rpaper_cache_walls,
            rpaper_set_templates,
            rpaper_set_walls,
            image_processing_change_contrast,
            image_processing_change_brigtness,
            image_processing_change_hue,
            image_processing_change_blur,
            image_processing_invert,
            image_processing_h_flip,
            image_processing_v_flip,
            image_processing_contrast,
            image_processing_brigtness,
            image_processing_hue,
            image_processing_blur,
            rwal_cache_dir,
            rwal_thumb_w,
            rwal_thumb_h,
            rwal_accent,
            rwal_clamp_min,
            rwal_clamp_max,
            displays,
            templates,
            variables,
        }
    }
}

fn get_string(value: Option<String>) -> Option<String> {
    match value {
        Some(val) => {
            if val.chars().nth(0).unwrap_or('-') == '-' {return None}
            Some(val)
        },
        None => None
    }
}

fn get_bool(value: Option<String>) -> Option<bool> {
    match value {
        Some(val) => {
            if val.chars().nth(0).unwrap_or('-') == '-' {return None}
            match val.parse() {
                Ok(val) => Some(val),
                Err(_) => None
            }
        },
        None => None
    }
}

fn get_u32(value: Option<String>) -> Option<u32> {
    match value {
        Some(val) => {
            if val.chars().nth(0).unwrap_or('-') == '-' {return None}
            match val.parse() {
                Ok(val) => Some(val),
                Err(_) => None
            }
        },
        None => None
    }
}

fn get_f32(value: Option<String>) -> Option<f32> {
    match value {
        Some(val) => {
            if val.chars().nth(0).unwrap_or('-') == '-' {return None}
            match val.parse() {
                Ok(val) => Some(val),
                Err(_) => None
            }
        },
        None => None
    }
}

fn get_i32(value: Option<String>) -> Option<i32> {
    match value {
        Some(val) => {
            if val.chars().nth(0).unwrap_or('-') == '-' {return None}
            match val.parse() {
                Ok(val) => Some(val),
                Err(_) => None
            }
        },
        None => None
    }
}
struct Request {
    image: Option<String>,
    cache: bool,
    w_set: bool,
    w_cache: bool,
    c_set: bool,
    c_cache: bool,
    displays: Option<Vec<Display>>,
    templates: Option<Vec<String>>,
    resize_alg: Option<String>,
    set_command: Option<String>,
    contrast: Option<i32>,
    brightness: Option<f32>,
    hue: Option<i32>,
    blur: Option<f32>,
    invert: bool,
    flip_h: bool,
    flip_v: bool,
    rwal_thumb: Option<String>,
    rwal_clamp: Option<String>,
    rwal_accent: Option<u32>,
    rwal_order: Option<String>,
    get_displays: bool,
    get_templates: bool,
    get_current_colorscheme: bool,
    get_image_ops: bool,
    get_rwal_params: bool,
    get_config: bool,
    get_w_cache: bool,
    get_c_cache: bool,
    w_cache_on_miss: bool,
    c_cache_on_miss: bool,
}

#[derive(Clone)]
struct Display {
    name: String,
    w: u32,
    h: u32,
    x: u32,
    y: u32,
}

pub trait Serialize {
    fn to_json(&self) -> String;
}

impl Serialize for u32 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl Serialize for i32 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl Serialize for f32 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl Serialize for String {
    fn to_json(&self) -> String {
        format!("\"{}\"", self)
    }
}

impl Serialize for bool {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl Serialize for Display {
    fn to_json(&self) -> String {
        format!(
            "{{name:{},w:{},h:{},x:{},y:{}}}",
            self.name, self.w, self.h, self.x, self.y,
        )
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn to_json(&self) -> String {
        format!(
            "[{}]",
            self.iter()
                .map(|el| { el.to_json() })
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl Serialize for Request {
    fn to_json(&self) -> String {
        let elements = vec![
            pack("image", self.image.clone()),
            pack("cache", Some(self.cache)),
            pack("w_set", Some(self.w_set)),
            pack("w_cache", Some(self.w_cache)),
            pack("c_set", Some(self.c_set)),
            pack("c_cache", Some(self.c_cache)),
            pack("displays", self.displays.clone()),
            pack("templates", self.templates.clone()),
            pack("resize_alg", self.resize_alg.clone()),
            pack("set_command", self.set_command.clone()),
            pack("contrast", self.contrast.clone()),
            pack("brightness", self.brightness.clone()),
            pack("hue", self.hue.clone()),
            pack("blur", self.blur.clone()),
            pack("invert", Some(self.invert)),
            pack("flip_h", Some(self.flip_h)),
            pack("flip_v", Some(self.flip_v)),
            pack("rwal_thumb", self.rwal_thumb.clone()),
            pack("rwal_clamp", self.rwal_clamp.clone()),
            pack("rwal_accent", self.rwal_accent.clone()),
            pack("rwal_order", self.rwal_order.clone()),
            pack("get_displays", Some(self.get_displays)),
            pack("get_templates", Some(self.get_templates)),
            pack(
                "get_current_colorscheme",
                Some(self.get_current_colorscheme),
            ),
            pack("get_image_ops", Some(self.get_image_ops)),
            pack("get_rwal_params", Some(self.get_rwal_params)),
            pack("get_config", Some(self.get_config)),
            pack("get_w_cache", Some(self.get_w_cache)),
            pack("get_c_cache", Some(self.get_c_cache)),
        ];

        format!(
            "{{{}}}",
            elements
                .iter()
                .filter_map(|el| {
                    if !el.is_empty() {
                        Some(el.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

fn pack<T: Serialize>(prefix: &str, value: Option<T>) -> String {
    if let Some(val) = value {
        return format!("{}:{}", prefix, val.to_json());
    }
    String::new()
}

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use rand::seq::SliceRandom;
use rand::thread_rng;

fn main() {
    if !Path::new(SOCKET_PATH).exists() {
        eprintln!("Daemon is not found. Is it running?");
        return;
    }

    if std::env::args()
        .collect::<Vec<String>>()
        .contains(&"--help".to_string())
    {
        println!("{HELP_MESSAGE}");
        return;
    }

    let mut req = Request::new(std::env::args().skip(1).collect());

    req.process();
}

struct Request {
    w_set: bool,
    w_cache: bool,
    c_set: bool,
    c_cache: bool,
    cache: bool,
    image: Option<String>,
    set_command: Option<String>,
    config_contrast: Option<i32>,
    config_brightness: Option<f32>,
    config_hue: Option<i32>,
    config_blur: Option<f32>,
    config_invert: bool,
    config_flip_h: bool,
    config_flip_v: bool,
    config_displays: Option<Vec<Display>>,
    config_templates: Option<Vec<String>>,
    config_resize_alg: Option<String>,
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

struct Display {
    name: String,
    w: u32,
    h: u32,
    x: u32,
    y: u32,
}

impl Request {
    fn new(input: Vec<String>) -> Self {
        // booleans
        let w_set = input.contains(&"-S".to_string());
        let w_cache = input.contains(&"-W".to_string()) && !w_set;
        let c_set = input.contains(&"-T".to_string());
        let c_cache = input.contains(&"-C".to_string()) && !c_set;
        let cache = input.contains(&"--cache".to_string());

        let config_invert = input.contains(&"--invert".to_string());
        let config_flip_h = input.contains(&"--fliph".to_string());
        let config_flip_v = input.contains(&"--flipv".to_string());

        let get_displays = input.contains(&"--get-displays".to_string());
        let get_templates = input.contains(&"--get-templates".to_string());
        let get_current_colorscheme = input.contains(&"--get-current-scheme".to_string());
        let get_image_ops = input.contains(&"--get-image-ops".to_string());
        let get_rwal_params = input.contains(&"--get-rwal-params".to_string());
        let get_config = input.contains(&"--get-config".to_string());
        let get_w_cache = input.contains(&"--get-w-cache".to_string());
        let get_c_cache = input.contains(&"--get-c-cache".to_string());
        let w_cache_on_miss = input.contains(&"--w-cache-on-miss".to_string());
        let c_cache_on_miss = input.contains(&"--c-cache-on-miss".to_string());

        // strings
        let image = get_value::<String>(&input, "-I");
        let set_command = get_value::<String>(&input, "--set-command");
        let config_resize_alg = get_value::<String>(&input, "--resize-alg");
        let rwal_thumb = get_value::<String>(&input, "--thumb");
        let rwal_clamp = get_value::<String>(&input, "--clamp");
        let rwal_order = get_value::<String>(&input, "--order");
        // nums
        let config_contrast = get_value::<i32>(&input, "--contrast");
        let config_brightness = get_value::<f32>(&input, "--brightness");
        let config_hue = get_value::<i32>(&input, "--hue");
        let config_blur = get_value::<f32>(&input, "--blur");
        let rwal_accent = get_value::<u32>(&input, "--accent");
        // arrays
        let config_displays = get_displays_value(&input, "--displays");
        let config_templates = get_templates_value(&input, "--templates");

        Self {
            w_set,
            w_cache,
            c_set,
            c_cache,
            cache,
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
            rwal_accent,
            rwal_order,
            get_displays,
            get_templates,
            get_current_colorscheme,
            get_image_ops,
            get_rwal_params,
            get_config,
            get_w_cache,
            get_c_cache,
            w_cache_on_miss,
            c_cache_on_miss,
        }
    }

    fn process(&mut self) {
        if self.get_displays {
            let _ = send("GET_DISPLAYS");
        }
        if self.get_templates {
            let _ = send("GET_TEMPLATES");
        }
        if self.get_current_colorscheme {
            let _ = send("GET_SCHEME");
        }
        if self.get_image_ops {
            let _ = send("GET_IMAGE_OPS");
        }
        if self.get_rwal_params {
            let _ = send("GET_RWAL_PARAMS");
        }
        if self.get_config {
            let _ = send("GET_CONFIG");
        }

        if let Some(image) = &self.image {
            if !is_dir(&get_absolute_path(image.to_string())) {
                let image_request = self.pack_image_request(&get_absolute_path(image.to_string()));
                let _ = send(&image_request);
                return;
            }

            let images = get_images_from_dir(&get_absolute_path(image.to_string()));

            if !self.cache {
                let image_request = self.pack_image_request(&select_random(images));
                let _ = send(&image_request);
                return;
            }

            let image_request = images
                .into_iter()
                .map(|image_path| self.pack_image_request(&image_path))
                .collect::<Vec<String>>()
                .join(";");
            let _ = send(&image_request);
        }
    }

    fn pack_image_request(&self, image_path: &str) -> String {
        let mut tags: Vec<String> = Vec::new();
        tags.push("IMAGE".to_string());
        tags.push(image_path.to_string());
        if self.get_w_cache {
            tags.push("GET_W_CACHE".to_string());
        }
        if self.get_c_cache {
            tags.push("GET_C_CACHE".to_string());
        }
        if self.w_cache_on_miss {
            tags.push("W_CACHE_ON_MISS".to_string());
        }
        if self.c_cache_on_miss {
            tags.push("C_CACHE_ON_MISS".to_string());
        }
        if self.w_set {
            tags.push("W_SET".to_string());
        }
        if self.w_cache {
            tags.push("W_CACHE".to_string());
        }
        if self.c_set {
            tags.push("C_SET".to_string());
        }
        if self.c_cache {
            tags.push("C_CACHE".to_string());
        }
        if let Some(contrast) = self.config_contrast {
            tags.push("CONFIG_CONTRAST".to_string());
            tags.push(contrast.to_string());
        }
        if let Some(brightness) = self.config_brightness {
            tags.push("CONFIG_BRIGHTNESS".to_string());
            tags.push(brightness.to_string());
        }
        if let Some(blur) = self.config_blur {
            tags.push("CONFIG_BLUR".to_string());
            tags.push(blur.to_string());
        }
        if let Some(hue) = self.config_hue {
            tags.push("CONFIG_HUE".to_string());
            tags.push(hue.to_string());
        }
        if self.config_invert {
            tags.push("CONFIG_INVERT".to_string());
        }
        if self.config_flip_h {
            tags.push("CONFIG_FLIP_H".to_string());
        }
        if self.config_flip_v {
            tags.push("CONFIG_FLIP_V".to_string());
        }
        if let Some(displays) = &self.config_displays {
            let displays_string = displays
                .into_iter()
                .map(|d| d.to_string())
                .collect::<Vec<String>>()
                .join(",");
            tags.push("CONFIG_DISPLAYS".to_string());
            tags.push(displays_string);
        }
        if let Some(templates) = &self.config_templates {
            let templates_string = templates
                .into_iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(",");
            tags.push("CONFIG_TEMPLATES".to_string());
            tags.push(templates_string);
        }
        if let Some(resize_alg) = &self.config_resize_alg {
            tags.push("CONFIG_RESIZE_ALG".to_string());
            tags.push(resize_alg.to_string());
        }
        if let Some(thumb) = &self.rwal_thumb {
            tags.push("RWAL_THUMB".to_string());
            tags.push(thumb.to_string());
        }
        if let Some(clamp) = &self.rwal_clamp {
            tags.push("RWAL_CLAMP".to_string());
            tags.push(clamp.to_string());
        }
        if let Some(accent) = &self.rwal_accent {
            tags.push("RWAL_ACCENT".to_string());
            tags.push(accent.to_string());
        }
        if let Some(order) = &self.rwal_order {
            tags.push("RWAL_ORDER".to_string());
            tags.push(order.to_string());
        }
        if let Some(set_command) = &self.set_command {
            tags.push("SET_COMMAND".to_string());
            tags.push(set_command.to_string());
        }
        tags.join("    ")
    }
}

impl Display {
    fn new(name: String, w: u32, h: u32, x: u32, y: u32) -> Self {
        Self { name, w, h, x, y }
    }
    fn to_string(&self) -> String {
        format!("{}:{}:{}:{}:{}", self.name, self.w, self.h, self.x, self.y,)
    }
}

fn send(message: &str) -> std::io::Result<()> {
    let message = format!("{}\n", message);

    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    let mut reader = BufReader::new(stream.try_clone()?);
    stream.write_all(message.as_bytes())?;
    let mut response = String::new();
    reader.read_line(&mut response)?;
    println!("{response}");
    Ok(())
}

fn get_value<T: std::str::FromStr>(list: &Vec<String>, prev_element: &str) -> Option<T> {
    for (i, el) in list.iter().enumerate() {
        if el == &prev_element {
            if i + 1 < list.len() {
                return match list[i + 1].parse::<T>() {
                    Ok(val) => Some(val),
                    Err(_) => None,
                };
            }
        }
    }
    None
}

fn get_displays_value(list: &Vec<String>, prev_element: &str) -> Option<Vec<Display>> {
    let mut displays: Vec<Display> = Vec::new();

    if let Some(raw_displays) = get_value::<String>(list, prev_element) {
        for raw_display in raw_displays.split(";") {
            let data: Vec<&str> = raw_display.split(":").collect();
            if data.len() != 5 {
                continue;
            }
            displays.push(Display::new(
                data[0].parse().unwrap_or("name".to_string()),
                data[1].parse().unwrap_or(0),
                data[2].parse().unwrap_or(0),
                data[3].parse().unwrap_or(0),
                data[4].parse().unwrap_or(0),
            ));
        }
    }

    match displays.is_empty() {
        true => None,
        false => Some(displays),
    }
}

fn get_templates_value(list: &Vec<String>, prev_element: &str) -> Option<Vec<String>> {
    let mut templates: Vec<String> = Vec::new();

    if let Some(raw_templates) = get_value::<String>(list, prev_element) {
        templates = raw_templates.split(";").map(|x| x.to_string()).collect();
    }

    match templates.is_empty() {
        true => None,
        false => Some(templates),
    }
}

fn is_dir(path: &str) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_dir())
        .unwrap_or(false)
}

fn select_random(strings: Vec<String>) -> String {
    let mut rng = thread_rng();

    if let Some(random_string) = strings.choose(&mut rng) {
        random_string.to_string()
    } else {
        panic!("Directory is empty")
    }
}

fn is_file_image(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "jpg" | "jpeg" | "webp" | "png" | "gif" | "bmp" | "tiff"
    )
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

const SOCKET_PATH: &str = "/tmp/rpaper-daemon";
const HELP_MESSAGE: &str = r#"+-----------------------------+-------------------------------------------------------+
|                             |                                                       |
|          argument           |                      description                      |
|                             |                                                       |
+-----------------------------+-------------------------------------------------------+
| -S                          | set wallpaper,                                        |
|                             | would cache wallpaper if it is not cached             |
|                             |                                                       |
| -W                          | cache wallpaper                                       |
|                             |                                                       |
| -T                          | apply color_scheme, would cache colors                |
|                             | if they are not cached                                |
|                             |                                                       |
| -C                          | cache colors                                          |
|                             |                                                       |
| --set-command <value>       | set different wallpaper set command                   |
+-----------------------------+-------------------------------------------------------+
| --contrast <value>          | change image contrast                                 |
|                             |                                                       |
| --brightness <value>        | change image brightness                               |
|                             |                                                       |
| --hue <value>               | change image hue                                      |
|                             |                                                       |
| --blur <value>              | change image blut                                     |
|                             |                                                       |
| --invert                    | invert image                                          |
|                             |                                                       |
| --fliph                     | flip image horizontaly                                |
|                             |                                                       |
| --flipv                     | flip image verticaly                                  |
|                             |                                                       |
| --displays <value>          | set displays wallpaper setted to params               |
|                             |     example:                                          |
|                             |     HDMI-A-1:1920:1080:0:0,DP-1:1080:1920:0:0         |
|                             |                                                       |
| --templates <value>         | set templates to be applied                           |
|                             |     example:                                          |
|                             |     path,anotherpath,anotherpath                      |
|                             |                                                       |
| --resize-alg <value>        | use different resize algorithm for wallpaper cache    |
+-----------------------------+-------------------------------------------------------+
| --thumb <value>             | set the dimensions of the image thumb                 |
|                             | from which the colors are taken                       |
|                             |     possible values:                                  |
|                             |     1-<int>X1-<int>                                   |
|                             |                                                       |
|                             |                                                       |
| --clamp <value>             | set the clamp range to pallete colors                 |
|                             |     possible values:                                  |
|                             |     1-255X1-255                                       |
|                             |                                                       |
|                             |                                                       |
| --count <value>             | set the number of colors to be generated              |
|                             |     work in progress..                                |
|                             |                                                       |
| --accent <value>            | set the accent color id                               |
|                             |     possible values:                                  |
|                             |     0-5                                               |
|                             |                                                       |
| --order <value>             | set the colorscheme order                             |
|                             |     possible values:                                  |
|                             |     h - order by hue                                  |
|                             |     s - order by saturation                           |
|                             |     v - order by brightness                           |
|                             |                                                       |
+-----------------------------+-------------------------------------------------------+
| -I <path/to/image>          | sends wallpaper to daemon                             |
|                             |                                                       |
| --cache                     | allow applying actions to all images from directory   |
|                             | if -I argument is a directory                         |
+-----------------------------+-------------------------------------------------------+
| --get-displays              | get loaded displays in json format                    |
|                             |                                                       |
| --get-current-scheme        | get current color scheme                              |
|                             |                                                       |
| --get-templates             | get loaded templates in json format                   |
|                             |                                                       |
| --get-image-ops             | get loaded image operations in json format            |
|                             |                                                       |
| --get-rwal-params           | get loaded rwal params in json format                 |
|                             |                                                       |
| --get-config                | get loaded config in json format                      |
|                             |                                                       |
| --get-w-cache               | get cached images paths                               |
|                             |                                                       |
| --get-c-cache               | get color pallete of image                            |
+-----------------------------+-------------------------------------------------------+"#;

// Socket calls
// +------------------------------+-------------------------------------------------------+
// |                              |                                                       |
// |          socket call         |                      description                      |
// |                              |                                                       |
// +------------------------------+-------------------------------------------------------+
// | W_SET                        | set wallpaper,                                        |
// |                              | would cache wallpaper if it is not cached             |
// |                              |                                                       |
// | W_CACHE                      | cache wallpaper                                       |
// |                              |                                                       |
// | C_SET                        | apply color_scheme,                                   |
// |                              | would cache colorscheme if it is not cached           |
// |                              |                                                       |
// | C_CACHE                      | cache colors                                          |
// |                              |                                                       |
// | SET_COMMAND                  | set different wallpaper set command                   |
// +------------------------------+-------------------------------------------------------+
// | CONFIG_CONTRAST <value>      | change image contrast                                 |
// |                              |                                                       |
// | CONFIG_BRIGHTNESS <value>    | change image brightness                               |
// |                              |                                                       |
// | CONFIG_HUE <value>           | change image hue                                      |
// |                              |                                                       |
// | CONFIG_BLUR <value>          | change image blut                                     |
// |                              |                                                       |
// | CONFIG_INVERT                | invert image                                          |
// |                              |                                                       |
// | CONFIG_FLIP_H                | flip image horizontaly                                |
// |                              |                                                       |
// | CONFIG_FLIP_V                | flip image verticaly                                  |
// |                              |                                                       |
// | CONFIG_DISPLAYS <value>      | set displays wallpaper setted to params               |
// |                              |     value example:                                    |
// |                              |         HDMI-A-1:1920:1080:0:0,DP-1:1080:1920:0:0     |
// |                              |                                                       |
// | CONFIG_TEMPLATES <value>     | set templates to be applied                           |
// |                              |     value example:                                    |
// |                              |         path,anotherpath,anotherpath                  |
// |                              |                                                       |
// | CONFIG_RESIZE_ALG <value>    | use different resize algorithm for wallpaper cache    |
// +------------------------------+-------------------------------------------------------+
// | RWAL_THUMB widthXheight      | set the dimensions of the image thumb                 |
// |                              | from which the colors are taken                       |
// |                              |                                                       |
// | RWAL_CLAMP minXmax           | set the clamp range to pallete colors                 |
// |                              |                                                       |
// | RWAL_COUNT colors_count      | set the number of colors to be generated              |
// |                              |                                                       |
// | RWAL_ACCENT accent_color_id  | set the accent color id                               |
// |                              |                                                       |
// | RWAL_ORDER order_char        | set the colorscheme ordering                          |
// |                              |                                                       |
// +------------------------------+-------------------------------------------------------+
// | IMAGE <path/to/image>        | sends wallpaper to daemon                             |
// |                              |                                                       |
// | loop of W_CACHE and IMAGE    | selects all images from dir and cache them            |
// |                              |                                                       |
// | loop of C_CACHE and IMAGE    | selects all images from dir and cache they colors     |
// |                              |                                                       |
// | loop of W_CACHE, C_CACHE     | selects all images from dir and cache them            |
// | and IMAGE                    | (colors and wallpapers)                               |
// +------------------------------+-------------------------------------------------------+
// | GET_DISPLAYS                 | get loaded displays in json format                    |
// |                              |                                                       |
// | GET_SCHEME                   | get current colorscheme in json format                |
// |                              |                                                       |
// | GET_TEMPLATES                | get loaded templates in json format                   |
// |                              |                                                       |
// | GET_IMAGE_OPS                | get loaded image operations in json format            |
// |                              |                                                       |
// | GET_RWAL_PARAMS              | get loaded rwal params in json format                 |
// |                              |                                                       |
// | GET_CONFIG                   | get loaded config in json format                      |
// |                              |                                                       |
// | GET_W_CACHE                  | get cached images paths                               |
// |                              | responds with json-like string                        |
// |                              | (automaticaly caches image if needed)                 |
// |                              |     respond examle:                                   |
// |                              |         [                                             |
// |                              |           {                                           |
// |                              |             "display": "HDMI-A-1",                    |
// |                              |             "path": "some/path/to/cache/image"        |
// |                              |           },                                          |
// |                              |           {                                           |
// |                              |             "display": "DP-1",                        |
// |                              |             "path": "some/path/to/cache/image"        |
// |                              |           }                                           |
// |                              |         ]                                             |
// |                              |                                                       |
// |                              |                                                       |
// | GET_C_CACHE                  | get color pallete of an image                         |
// |                              | responds with json-like string                        |
// |                              | (automaticaly caches colorscheme if needed)           |
// |                              |     respond examle:                                   |
// |                              |         [                                             |
// |                              |           "pallete color 0 in HEX",                   |
// |                              |           "pallete color 1 in HEX",                   |
// |                              |           "pallete color 2 in HEX",                   |
// |                              |           "pallete color 3 in HEX",                   |
// |                              |           "pallete color 4 in HEX",                   |
// |                              |           "pallete color 5 in HEX",                   |
// |                              |           "pallete color 6 in HEX",                   |
// |                              |           "pallete color 7 in HEX",                   |
// |                              |           "pallete color 8 in HEX",                   |
// |                              |           "pallete color 9 in HEX",                   |
// |                              |           "pallete color 10 in HEX",                  |
// |                              |           "pallete color 11 in HEX",                  |
// |                              |           "pallete color 12 in HEX",                  |
// |                              |           "pallete color 13 in HEX",                  |
// |                              |           "pallete color 14 in HEX",                  |
// |                              |           "pallete color 15 in HEX"                   |
// |                              |         ]                                             |
// +------------------------------+-------------------------------------------------------+
// Each socket call and value must be splitted by four spaces "    "

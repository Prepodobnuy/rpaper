use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use rand::seq::{IndexedRandom, SliceRandom};
use rand::rng;

use common::Request;

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

    let req = Request::from_args(std::env::args().skip(1).collect());
    let serialized = serde_json::to_string(&req).unwrap();
    let _ = send(&serialized);
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

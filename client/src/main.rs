use std::os::unix::net::UnixStream;
use std::io::{Write, BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::fs;
use std::str::FromStr;

use rand::seq::SliceRandom;
use rand::thread_rng;


const REQUIRED_KEYWORDS: [&str; 6] = ["IMAGE", "GET_DISPLAYS", "GET_TEMPLATES", "GET_IMAGE_OPS", "GET_RWAL_PARAMS", "GET_CONFIG"];
const RECEIVE_KEYWORDS: [&str; 5] = ["GET_DISPLAYS", "GET_TEMPLATES", "GET_IMAGE_OPS", "GET_RWAL_PARAMS", "GET_CONFIG"];

fn send(message: String) -> std::io::Result<()> {
    let mut message = format!("{}\n", message);

    if RECEIVE_KEYWORDS.iter().any(|&keyword| message.contains(keyword)) {
        for &keyword in RECEIVE_KEYWORDS.iter() {
            let mut stream = UnixStream::connect(SOCKET_PATH)?;
            let mut reader = BufReader::new(stream.try_clone()?);
            if message.contains(keyword) {
                stream.write_all(format!("{}\n", keyword).as_bytes())?;
                let mut response = String::new();
                reader.read_line(&mut response)?;
                println!("{}", response);
                message = message.replace(format!("    {}", keyword).as_str(), "")
                    .replace(format!("{}    ", keyword).as_str(), "");
            }
        }
    }
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    let mut reader = BufReader::new(stream.try_clone()?);
    stream.write_all(message.as_bytes())?;
    let mut response = String::new();
    reader.read_line(&mut response)?;

    Ok(())
}

fn is_dir(path: &str) -> bool {
    fs::metadata(path).map(|meta| meta.is_dir()).unwrap_or(false)
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).map(|meta| meta.is_file()).unwrap_or(false)
}

fn select_random(strings: Vec<String>) -> String {
    let mut rng = thread_rng();

    if let Some(random_string) = strings.choose(&mut rng) {
        random_string.to_string()
    } else {
        panic!("Directory is empty")
    }
}

fn replace_arguments(args: &Vec<String>) -> Result<String, String> {
    let mut args: Vec<String> = args.clone();
    
    if let Some(pos) = args.iter().position(|arg| arg == "-I") {
        if pos + 1 < args.len() {
            let next_element = &args[pos + 1];
            if is_dir(&next_element) {
                args[pos + 1] = get_absolute_path(select_random(get_images_from_dir(&next_element)));
            } 
            else if is_file(&next_element) {
                args[pos + 1] = get_absolute_path(next_element.to_string());
            }
        }
    }

    let mut result = args.join("    ")
        .replace("-S", "W_SET")
        .replace("-W", "W_CACHE")
        .replace("-T", "C_SET")
        .replace("-C", "C_CACHE")
        .replace("--set-command", "SET_COMMAND")
        .replace("-I", "IMAGE")
        .replace("--contrast", "CONFIG_CONTRAST")
        .replace("--brightness", "CONFIG_BRIGHTNESS")
        .replace("--hue", "CONFIG_HUE")
        .replace("--blur", "CONFIG_BLUR")
        .replace("--invert", "CONFIG_INVERT")
        .replace("--fliph", "CONFIG_FLIP_H")
        .replace("--flipv", "CONFIG_FLIP_V")
        .replace("--displays", "CONFIG_DISPLAYS")
        .replace("--templates", "CONFIG_TEMPLATES")
        .replace("--resize-alg", "CONFIG_RESIZE_ALG")
        .replace("--thumb", "RWAL_THUMB")
        .replace("--clamp", "RWAL_CLAMP")
        .replace("--count", "RWAL_COUNT")
        .replace("--accent", "RWAL_ACCENT")
        .replace("--get-displays", "GET_DISPLAYS")
        .replace("--get-templates", "GET_TEMPLATES")
        .replace("--get-image-ops", "GET_IMAGE_OPS")
        .replace("--get-rwal-params", "GET_RWAL_PARAMS")
        .replace("--get-config", "GET_CONFIG");

    if result.contains("W_SET") && result.contains("W_CACHE") {
        result = result.replace("W_CACHE    ", "");
        result = result.replace("    W_CACHE", "");
    }
    if result.contains("C_SET") && result.contains("C_CACHE") {
        result = result.replace("C_CACHE    ", "");
        result = result.replace("    C_CACHE", "");
    }
    if !REQUIRED_KEYWORDS.iter().any(|&keyword| result.contains(keyword)) {
        return Err("Missing -I".to_string());
    }
    Ok(result)
}

fn is_file_image(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "jpg" | "jpeg" | "webp" | "png" | "gif" | "bmp" | "tiff"
    )
}

fn get_absolute_path(path: String) -> String {
    let Ok(path) = PathBuf::from_str(&path);
    
    return path.canonicalize().unwrap_or_else(|_| path)
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
            res.extend(get_images_from_dir(
                &get_absolute_path(entry.path().to_string_lossy().to_string())
            ))
        } else if file_type.is_file() {
            if let Some(extension) = entry.path().extension() {
                if is_file_image(extension.to_str().unwrap_or("")) {
                    res.push(
                        get_absolute_path(entry.path().to_string_lossy().to_string())
                    )
                }
            }
        }
    }
    res
}

fn cache_directory(args: &Vec<String>, argument: &str, prefix: &str) {
    let dir = args.iter()
        .position(|x| x == argument)
        .and_then(|pos| args.get(pos + 1))
        .unwrap();
    
    let images = get_images_from_dir(dir);
    
    let _ = send(
        images.into_iter()
            .map(|x| {
                format!("IMAGE    {}    {}", x, prefix)
            })
            .collect::<Vec<String>>()
            .join(";")
    );
}

fn main() {
    if !Path::new(SOCKET_PATH).exists() {
        eprintln!("Daemon is not found. Is it running?");
        return;
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.contains(&String::from("--help")) {
        println!("{}", HELP_MESSAGE);
        return;
    }
    if args.contains(&String::from("--cache")) {
        cache_directory(&args, "--cache", "W_CACHE");
        return;
    }
    if args.contains(&String::from("--colors")) {
        cache_directory(&args, "--colors", "C_CACHE");
        return;
    }
    if args.contains(&String::from("--cache-all")) {
        cache_directory(&args, "--cache-all", "C_CACHE    W_CACHE");
        return;
    }
    match replace_arguments(&args) {
        Ok(message) => {
            let _ = send(message);
        },
        Err(message) => {
            eprintln!("Error {}\n--help for more commands", message);
        },
    }
}

const SOCKET_PATH: &str = "/tmp/rpaper-daemon";
const HELP_MESSAGE: &str = 
r#"+-----------------------------+-------------------------------------------------------+
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
|                             | example: HDMI-A-1:1920:1080:0:0,DP-1:1080:1920:0:0    |
|                             |                                                       |
| --templates <value>         | set templates to be applied                           |
|                             | example: path,anotherpath,anotherpath                 |
|                             |                                                       |
| --resize-alg <value>        | use different resize algorithm for wallpaper cache    |
+-----------------------------+-------------------------------------------------------+
| --thumb <value>             | set the dimensions of the image thumb                 |
|                             | from which the colors are taken                       |
|                             |                                                       |
| --clamp <value>             | set the clamp range to pallete colors                 |
|                             |                                                       |
| --count <value>             | set the number of colors to be generated              |
|                             |                                                       |
| --accent <value>            | set the accent color id                               |
+-----------------------------+-------------------------------------------------------+
| -I <path/to/image>          | sends wallpaper to daemon                             |
|                             |                                                       |
| --cache <dir>               | selects all images from dir and cache them            |
|                             |                                                       |
| --colors <dir>              | selects all images from dir and cache they colors     |
|                             |                                                       |
| --cache-all <dir>           | selects all images from dir and cache them            |
|                             | (colors and wallpapers)                               |
+-----------------------------+-------------------------------------------------------+
| --get-displays              | get loaded displays in json format                    |
|                             |                                                       |
| --get-templates             | get loaded templates in json format                   |
|                             |                                                       |
| --get-image-ops             | get loaded image operations in json format            |
|                             |                                                       |
| --get-rwal-params           | get loaded rwal params in json format                 |
|                             |                                                       |
| --get-config                | get loaded config in json format                      |
|                             |                                                       |
| --is-cached <path/to/image> | get cached images paths                               |
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
// | C_SET                        | apply color_scheme, would cache colors                |
// |                              | if they are not cached                                |
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
// |                              | example: HDMI-A-1:1920:1080:0:0,DP-1:1080:1920:0:0    |
// |                              |                                                       |
// | CONFIG_TEMPLATES <value>     | set templates to be applied                           |
// |                              | example: path,anotherpath,anotherpath                 |
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
// | GET_TEMPLATES                | get loaded templates in json format                   |
// |                              |                                                       |
// | GET_IMAGE_OPS                | get loaded image operations in json format            |
// |                              |                                                       |
// | GET_RWAL_PARAMS              | get loaded rwal params in json format                 |
// |                              |                                                       |
// | GET_CONFIG                   | get loaded config in json format                      |
// |                              |                                                       |
// | IS_CACHED <path/to/image>    | get cached images paths                               |
// +------------------------------+-------------------------------------------------------+
// Each socket call and value must be splitted by four spaces "    "
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
    let mut stream = UnixStream::connect(SOCKET_PATH)?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let message = format!("{}\n", message);

    if RECEIVE_KEYWORDS.iter().any(|&keyword| message.contains(keyword)) {
        for &keyword in RECEIVE_KEYWORDS.iter() {
            if message.contains(keyword) {
                stream.write_all(format!("{}\n", keyword).as_bytes())?;
                let mut response = String::new();
                reader.read_line(&mut response)?;
                println!("{}", response);
            }
        }
    } else {
        stream.write_all(message.as_bytes())?;
        let mut response = String::new();
        reader.read_line(&mut response)?;
    }

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
r#"+----------------------+-------------------------------------------------------+------------------------------+
|                      |                                                       |                              |
|       argument       |                      description                      |          socket call         |
|                      |                                                       |                              |
+----------------------+-------------------------------------------------------+------------------------------+
| -S                   | set wallpaper,                                        | W_SET                        |
|                      | would cache wallpaper if it is not cached             |                              |
|                      |                                                       |                              |
| -W                   | cache wallpaper                                       | W_CACHE                      |
|                      |                                                       |                              |
| -T                   | apply color_scheme, would cache colors                | C_SET                        |
|                      | if they are not cached                                |                              |
|                      |                                                       |                              |
| -C                   | cache colors                                          | C_CACHE                      |
|                      |                                                       |                              |
| --set-command <value>| set different wallpaper set command                   | SET_COMMAND                  |
+----------------------+-------------------------------------------------------+------------------------------+
| --contrast <value>   | change image contrast                                 | CONFIG_CONTRAST <value>      |
|                      |                                                       |                              |
| --brightness <value> | change image brightness                               | CONFIG_BRIGHTNESS <value>    |
|                      |                                                       |                              |
| --hue <value>        | change image hue                                      | CONFIG_HUE <value>           |
|                      |                                                       |                              |
| --blur <value>       | change image blut                                     | CONFIG_BLUR <value>          |
|                      |                                                       |                              |
| --invert             | invert image                                          | CONFIG_INVERT                |
|                      |                                                       |                              |
| --fliph              | flip image horizontaly                                | CONFIG_FLIP_H                |
|                      |                                                       |                              |
| --flipv              | flip image verticaly                                  | CONFIG_FLIP_V                |
|                      |                                                       |                              |
| --displays <value>   | set displays wallpaper setted to params               | CONFIG_DISPLAYS <value>      |
|                      | example: HDMI-A-1:1920:1080:0:0,DP-1:1080:1920:0:0    |                              |
|                      |                                                       |                              |
| --templates <value>  | set templates to be applied                           | CONFIG_TEMPLATES <value>     |
|                      | example: path,anotherpath,anotherpath                 |                              |
|                      |                                                       |                              |
| --resize-alg <value> | use different resize algorithm for wallpaper cache    | CONFIG_RESIZE_ALG <value>    |
+----------------------+-------------------------------------------------------+------------------------------+
| --thumb <value>      | set the dimensions of the image thumb                 | RWAL_THUMB widthXheight      |
|                      | from which the colors are taken                       |                              |
|                      |                                                       |                              |
| --clamp <value>      | set the clamp range to pallete colors                 | RWAL_CLAMP minXmax           |
|                      |                                                       |                              |
| --count <value>      | set the number of colors to be generated              | RWAL_COUNT colors_count      |
|                      |                                                       |                              |
| --accent <value>     | set the accent color id                               | RWAL_ACCENT accent_color_id  |
+----------------------+-------------------------------------------------------+------------------------------+
| -I <path/to/image>   | sends wallpaper to daemon                             | IMAGE <path/to/image>        |
|                      |                                                       |                              |
| --cache <dir>        | selects all images from dir and cache them            | loop of W_CACHE and IMAGE    |
|                      |                                                       |                              |
| --colors <dir>       | selects all images from dir and cache they colors     | loop of C_CACHE and IMAGE    |
|                      |                                                       |                              |
| --cache-all <dir>    | selects all images from dir and cache them            | loop of W_CACHE, C_CACHE     |
|                      | (colors and wallpapers)                               | and IMAGE                    |
+----------------------+-------------------------------------------------------+------------------------------+
| --get-displays       |                                                       | GET_DISPLAYS                 |
|                      |                                                       |                              |
| --get-templates      |                                                       | GET_TEMPLATES                |
|                      |                                                       |                              |
| --get-image-ops      |                                                       | GET_IMAGE_OPS                |
|                      |                                                       |                              |
| --get-rwal-params    |                                                       | GET_RWAL_PARAMS              |
|                      |                                                       |                              |
| --get-config         |                                                       | GET_CONFIG                   |
+----------------------+-------------------------------------------------------+------------------------------+"#;

// socket call examples
// W_SET C_SET IMAGE <path/to/image> // set wallpapers and apply templats of image
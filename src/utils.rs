use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::fs::File;
use std::io::Read;

use serde_json::Value;

use crate::Path;
use crate::config::ImageOperations;

fn add_home_path_to_string(path: &str) -> String {
    let home_dir = match env::var_os("HOME") {
        Some(dir) => PathBuf::from(dir),
        _none => {
            eprintln!("Error: HOME environment variable is not set.");
            std::process::exit(1);
        }
    };

    return home_dir.join(path).into_os_string().into_string().unwrap();
}

pub fn parse_path(path: &str) -> String {
    if &path[0..1] == "~" {
        return add_home_path_to_string(&path[2..]);
    }
    String::from(path)
}

pub fn parse_command(command: &str, image_path: &str, original_image_path: &str, display: &str) -> String {
    //let res = command.to_owned() + " > /dev/null";
    return command
        .replace("{image}", image_path)
        .replace("{default_image}", original_image_path)
        .replace("{display}", display);
}

pub fn system(command: &str) {
    let mut child = Command::new("bash")
        .args(["-c", &command])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn command");

    let _exit_status = child.wait().expect("Failed to wait for command");
}

pub fn spawn(command: &str) {
    Command::new("bash")
        .args(["-c", &command])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Err");
}

pub fn get_image_name(image_path: &str) -> String {
    let path = Path::new(image_path);
    if let Some(file_name) = path.file_name() {
        if let Some(name) = file_name.to_str() {
            return String::from(name);
        }
    }
    String::from(image_path)
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

pub fn help_message() { // TODO rewrite help message. Almost everything written here is deprecated LOL
    let help_message = r#"Usage:
  rpaper <path/to/dir/with/images>|<path/to/image> -flag, --second-flag
--help                                          - display this message

--temp-path <path/to/template>                  - overwrite path to templates
--vars-path <path/to/variables>                 - overwrite path to color variables
--cache-dir <path/to/cache-dir>                 - overwrite path to cached wallpapers
--color-scheme-file <path/to/color-scheme-file> - overwrite path to color scheme file
--set-wallpaper-command <set-wallpaper-command> - overwrite command to set wallpapers
  keywords:
    {display} -> display wallpaper set to
    {image}   -> path of image file setted to display
  
  examples:
    swww -i {image} -o {display}
    swaybg -i {image} -o {display}

--resize-backend <resize-backend>               - overwrite resize algorithm
  posible values:
   Nearest
   Triangle
   CatmllRom
   Gaussian
   Lanczos3

--cache-colorscheme <true/false>                - idk what to write here lol
--apply-templates <true/false>
--cache-wallpapers <true/false>
--set-wallpaper <true/false>

--change-contrast <true/false>
--change-brightness <true/false>
--change-hue <true/false>
--change-blur <true/false>

--contrast-value <int>
  possible values:
    from -255 to 255
--brightness-value <int>
  possible values:
    from -255 to 255
--hue-value <int>
  possible values:
    from 0 to 360
--blur-value <float>
  possible values:
    any

--apply-inversion <true/false>
--apply-h-flip <true/false>
--apply-v-flip <true/false>

--rwal-cache-dir <path/to/rwal/cache/dir>
--rwal-thumb-width
--rwal-thumb-height
--rwal-thumb-width
--rwal-accent-color
--rwal-clamp-min
--rwal-clamp-max

--displays <displays>
  posible values:
    HDMI-A-1:1920:1080:0:0,DP-A-1:1920:1080:0:0
    DISPLAY_NAME:DISPLAY_WIDTH:DISPLAY_HEIGHT:DISPLAY_X:DISPLAY_Y,ANOTHER_DISPLAY:ANOTHER_DISPLAY_WIDTH..."#;
    println!("{}", help_message);
}

pub fn read_data(data_path: &str) -> Value {
    let mut file = File::open(data_path).unwrap();
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).unwrap();

    let data: Value = serde_json::from_str(&json_data).unwrap();

    data
}

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
rpaper <path/to/image> 
rpaper <path/to/dir>

Flags: 

--vars-path <path/to/color/variables>

--cache-dir <path/to/cache/dir>

--wall-command "some wallpaper backend" 
               "swaybg -o {display} -i {image}"
               display -> display name
               image   -> cached image path

--resize-algorithm "algorithm"
                   CatmullRom
                   Gaussian
                   Lanczos3
                   Nearest
                   Triangle

--cache-color-scheme true|false
--cache-wallpaper true|false
--set-templates true|false
--set-wallpaper true|false

--change-contrast true|false
--change-brightness true|false
--change-hue true|false
--change-blur true|false
--invert true|false
--h-flip true|false
--v-flip true|false
--contrast <float number>
--brightness <float number>
--hue <integer number>
--blur <float number>

--r-cache-dir <path/to/cache/directory>
--thumb-w <int number larger than 0>
--thumb-h <int number larger than 0>
--accent <int number larger than 0>
--clamp-min <float number larger than 0>
--clamp-max <float number larger than 0>

--displays "displays"
           DISPLAY_NAME:DISPLAY_WIDTH:DISPLAY_HEIGHT:DISPLAY_X:DISPLAY_Y,ANOTHER_DISPLAY_NAME...
           HDMI-A-1:1920:1080:0:0,DP-A-1:1920:1080:0:0

--templates "templates"
            /path/to/template,/path/to/template,/path/to/template

--variables "variables"
            work in progress :p


"#;
    println!("{}", help_message);
}

pub fn read_data(data_path: &str) -> Value {
    let mut file = File::open(data_path).unwrap();
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).unwrap();

    let data: Value = serde_json::from_str(&json_data).unwrap();

    data
}

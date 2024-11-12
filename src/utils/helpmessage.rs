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
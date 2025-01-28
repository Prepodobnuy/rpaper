## I STRONGLY ADVISE YOU TO WAIT FOR THE RELEASE OF THE PROGRAM AS ALL THIS INFORMATION MAY LOSE RELEVANCE AT ANY MOMENT
## It is already lost relevance xd. Wait for update
## Configure
## config.json file
### parameters:
|parameter|type|description|possible values|default value|
|---|---|---|---|---|
|displays|List|list of displays|
|temp_path|String|path to templates.json|path|~/.config/rpaper/templates.json
|vars_path|String|path to color_variables.json|path|~/.config/rpaper/color_variables.json
|cache_dir|String|path to splited wallpapers|path|~/.cache/rpaper/Wallpapers
|scheme_file|String|path to color scheme file|path|~/.cache/rpaper/rwal/colors
|wall_command|String|command to set wallpapers to displays. supports formatting where {display} is a name of display and {image} is a path to splitted wallpaper|swaybg/swww command|swaybg -o {display} -i {image}
|resize_algorithm|String|image resize algorithm|Nearest CatmullRom Gaussian Lanczos3 Triangle|Triangle|
|cache_scheme|Boolean|responsible for creating color pallete|true/false|true
|set_templates|Boolean|responsible for changing configs|true/false|true
|cache_walls|Boolean|responsible for splitting wallpaper|true/false|true
|set_walls|Boolean|responsible for setting wallpaper|true/false|true
|imgp_change_contrast|Boolean|responsible for applying contrast to wallpaper|true/false|false
|imgp_change_brightness|Boolean|responsible for applying brightness to wallpaper|true/false|false
|imgp_change_huerotate|Boolean|responsible for applying huerotate to wallpaper|true/false|false
|imgp_change_blur|Boolean|responsible for applying blur to wallpaper|true/false|false
|imgp_contrast|Float|value to apply|-255 to 255|0
|imgp_brightness|Float|value to apply|-255 to 255|0
|imgp_huerotate|Number|value to apply|-255 to 255|0
|imgp_blur|Number|value to apply|0 to 255|0
|imgp_invert|Boolean|inverting wallpaper colors|true/false|false
|imgp_flip_h|Boolean|horizontaly flip wallpaper|true/false|false
|imgp_flip_v|Boolean|verticaly flip wallpaper|true/false|false
|rwal_cache_dir|String|path to cached color palettes|path|~/.cache/rpaper/rwal
|rwal_thumb_w|Number|width of thumbed wallpaper|2-any|100
|rwal_thumb_h|Number|height of thumbed wallpaper|2-any|100
|rwal_accent_color|Number|color from pallete from which bg and fg colors are generated|0-5|2
|rwal_clamp_min|Float|minimal brightness of colors in pallete|0-255|100
|rwal_clamp_max|Float|maximum brightness of colors in pallete|0-255|100
### displays:
|parameter|type|description|possible values|default value|
|---|---|---|---|---|
|name|String|name of display|any|none|
|width|Number|width of display|any|none|
|height|Number|height of display|any|none|
|margin-left|Number|x position of display|any|none|
|margin-top|Number|y position of display|any|none|
### example:
```json
{
  "displays": [
    {
      "name": "HDMI-A-1",
      "width": 1920,
      "height": 1080,
      "margin-left": 1080,
      "margin-top": 305
    },
    {
      "name": "DP-1",
      "width": 1080,
      "height": 1920,
      "margin-left": 0,
      "margin-top": 0
    }
  ],
  "temp_path": "~/.config/rpaper/templates.json",
  "vars_path": "~/.config/rpaper/color_variables.json",
  "cache_dir": "~/.cache/rpaper/Wallpapers",
  "scheme_file": "~/.cache/rpaper/rwal/colors",
  "wall_command": "swaybg -o {display} -i {image}",
  "resize_algorithm": "Lanczos3",

  "cache_scheme": true,
  "cache_walls": true,
  "set_templates": true,
  "set_walls": true,

  "imgp_change_contrast": false,
  "imgp_contrast": -15,
  "imgp_change_brightness": false,
  "imgp_brightness": -60,
  "imgp_change_huerotate": false,
  "imgp_huerotate": 200,
  "imgp_change_blur": false,
  "imgp_blur": 7.5,
  "imgp_invert": false,
  "imgp_flip_h": false,
  "imgp_flip_v": false,

  "rwal_cache_dir": "~/.cache/rpaper/rwal",
  "rwal_thumb_w": 210,
  "rwal_thumb_h": 210,
  "rwal_accent_color": 4,
  "rwal_clamp_min": 140.0,
  "rwal_clamp_max": 170.0
}
```
## templates.json
### parameters
|parameter|type|description|
|---|---|---|
|template_path|String|path to template config file|
|config_path|String|path to config config file|
|use_quotes|Boolean|param that wraps color in "": #001122 to "#001122"|
|use_sharps|Boolean|param that uses sharp before color: 001122 to #001122|
|opacity|String|param that is added after color: 001122 to 00112233|
|command|String|param that is execute command before&after changing config|
```
to use before-command  use || in command: killall waybar||waybar&
```
### example:
```json
[
  {
    "template_path": "~/.my_script_files/wallpaperpy_templates/alacritty.toml",
    "config_path": "~/.config/alacritty/alacritty.toml",
    "use_quotes": true,
    "use_sharps": true,
    "opacity": "",
    "command": ""
  },
  {
    "template_path": "~/.my_script_files/wallpaperpy_templates/waybar.css",
    "config_path": "~/.config/waybar/style.css",
    "use_quotes": false,
    "use_sharps": true,
    "opacity": "",
    "command": "killall waybar"
  }
]
```
## color_variables.json
### parameters
|parameter|type|description|
|---|---|---|
|name|String|name of color variable|
|value|Number|value of color from palette|
|brightness|Number|value which would be added to r,g,b of color|
|inverted|Boolean|invert color|
to create multiple brightness colors use {br} in color name. This would generate 20 more colors.
- Example:
```
"name": "(bg{br})" = (bgd1), (bgd2), (bgd3), (bgd4), (bgl1), (bgl2), (bgl3), (bgl4) etc.
```
d/l means darker/lighter, number is a number of tens added to the brightness of the original color.

### example:
```json
[
  {
    "name": "(bg{br})",
    "value": 15,
    "brightness": 10,
    "inverted": false
  },
  {
    "name": "(text)",
    "value": 0,
    "brightness": 0,
    "inverted": false
  },
  {
    "name": "(primary{br})",
    "value": 6,
    "brightness": 0
  }
]
```

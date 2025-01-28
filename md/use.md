## How to use rpaper
- ### Install it
I hope you have already done this lmao
- ### Copy default config file
```sh
cp /configs/config.json ~/.config/rpaper/config.json
```
- ### Paste your display params in config file
```
  "displays": [
    {
      "name": "HDMI-A-1",
      "w": 1920,
      "h": 1080,
      "x": 1080,
      "y": 100
    },
    {
      "name": "DP-1",
      "w": 1080,
      "h": 1920,
      "x": 0,
      "y": 0
    }
 
```
You will need to replace HDMI-A-1 and DP-1 to your displays
 - ### Start daemon
```
rpaper-daemon
```
 - ### Use client
```
rpaper --help
```
 - ### Most common rpaper usage
```
rpaper -I </path/to/image/or/to/folder/with/a/lot/of/images> -S -T
```
This command would set passed image as wallpaper and create color pallete from it. For now for more cawai baka sigma interface you can use [rpaper-rofi](https://github.com/Prepodobnuy/rpaper-rofi) python script. Peace <3

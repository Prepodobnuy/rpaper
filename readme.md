![rpaper logo](rpaper.png)

## About
Rpaper is needed to color programs in your cool and modern GNU/Linux distribution.
To color rpaper uses templates, human readable meta-configuration files. (configs of configs lol)
Templates contain the original configuration file, some parameters and ability to paste colors in declared variables.

<details>
  <summary><strong>Dynamic colortheme example</strong></summary>

```
// Dynamic color variables grabs colors from your wallpaper
// Syntax:
// Color(arg1, arg2, arg3, arg4) <- Dynamic color call 
// arg1 <- name of color dynamic variable (used to replace itself with color)
// arg2 <- palette color index (0:15)
// arg3 <- brightness modifier (-255:255)
// arg4 <- color inversion (true|false)

Color((bg{br}), 0) // it is not necessary to paste all 4 argument, 2 is enough
Color((fg{br}), 15)
Color((Obg{br}), 15)
Color((Ofg{br}), 0)
Color((pr{br}), 2, 20)

Color((0{br}), 0, 20)
Color((1{br}), 1, 20)
Color((2{br}), 2, 20)
Color((3{br}), 3, 20)
Color((4{br}), 4, 20)
Color((5{br}), 5, 20)
Color((6{br}), 6, 20)
Color((7{br}), 7, 20)

Color((8{br}), 0, -20)
Color((9{br}), 1, -20)
Color((10{br}), 2, -20)
Color((11{br}), 3, -20)
Color((12{br}), 4, -20)
Color((13{br}), 5, -20)
Color((14{br}), 6, -20)
Color((15{br}), 7, -20)
```
</details>

<details>
  <summary><strong>Static colortheme example</strong></summary>

```
// Static color variables are independend from your wallpaper
// To declare static color you can use HEX or RGB functions
// Syntax
// HEX(arg1, hex) <- Static color call
// RGB(arg1, r, g, b) <- also static color call

HEX((bg{br}), 282828)
HEX((fg{br}), ebdbb2)
HEX((Obg{br}), ebdbb2)
HEX((Ofg{br}), 282828)
HEX((pr{br}), d79921)

HEX((0{br}), 282828)
HEX((1{br}), cc241d)
HEX((2{br}), 98971a)
HEX((3{br}), d79921)
HEX((4{br}), 458588)
HEX((5{br}), b16286)
HEX((6{br}), 689d6a)
HEX((7{br}), a89984)

HEX((8{br}),  928374)
HEX((9{br}),  fb4934)
HEX((10{br}), b8bb26)
HEX((11{br}), fabd2f)
HEX((12{br}), 83a598)
HEX((13{br}), d3869b)
HEX((14{br}), 8ec07c)
HEX((15{br}), ebdbb2)
```
</details>

<details>
  <summary><strong>Usage example</strong></summary>

```
// template file
Path(~/.config/foot/foot.ini)          // path to paste modified config
Format({HEX})                          // format of color to paste color variables in
Include(~/path/to/your/amazing/colors) // replaces itself with a contaiment of file
                                       // usefull to store colorvars in separate place
//Color(NAME, 0)
//HEX(NAME, 000000)
//RGB(NAME, 0, 0, 0)

ExecBefore(echo "Hello, World!")       // shell command to execute before color pasting
ExecBefore(echo "Hello, World! 2")     // you can use multiple

ExecAfter(echo "Bye, World ;(")        // shell command to execute after color pasting
ExecAfter(echo "Bye, World ;(")        // you can also use multiple

// config keyword, all after it would be modified with rpaper
// DO NOT WRITE COMMENTS AFTER IT
[config]
font=Ubuntu Mono Nerd Font:size=12

[colors]
background= (bg)
foreground= (fg)
regular0=   (0)
regular1=   (1)
regular2=   (2)
regular3=   (3)
regular4=   (4)
regular5=   (5)
regular6=   (6)
regular7=   (7)
bright0=    (8)
bright1=    (9)
bright2=    (10)
bright3=    (11)
bright4=    (12)
bright5=    (13)
bright6=    (14)
bright7=    (15)
```
</details>

Also rpaper can set one wallpaper to multiple displays and grab color palette from it just like pywal.

Rpaper is still **under active development**, so don't be surprised that after the update your pc will burn and configuration files will be sent to Microsoft servers.
## Goals
- make it easies to rice linux
- make it easies to manage dotfiles
## Features
- uniqueness (analogues of the program are bash scripts with pyval which are very long and tedious to write)
- much faster than bash scripts with pyval
- configurable color pallete generator
- easy to use template system
- written on rust
- idk
## How to
- [install](https://github.com/Prepodobnuy/rpaper/blob/main/md/install.md)
- [configure](https://github.com/Prepodobnuy/rpaper/blob/main/md/configure.md)
- [use](https://github.com/Prepodobnuy/rpaper/blob/main/md/use.md)
## Contributing
feel free
## Todo
- ~~abandon pywal~~
- ~~add client arguments~~
- ~~implement sane --help for client~~
- ~~readme~~
- ~~daemon unix socket listener~~
- ~~restructure rwal code~~
- (WIP) implement semantic (red, green, yellow, blue, purple, aqua) palette ordering
- restructure listener architecture (use json instead of shitty tags)
- daemon http listener
- client http
- daemon rpc listener
- client rpc
- debloat dependencies
- video support using [mpvpaper](https://github.com/GhostNaN/mpvpaper)
- own wayland wallpaper setter (abandon swaybg/swww/etc)
- add diferent image setting modes (fit/fill/crop/etc)
- implement sane --help for daemon
- implement sane logger for daemon
- write wiki
- add animated files(.gif/.png/.webp) splitting support (currently they split without animation)
- maybe abandon json format for config
- release
## Thanks to
- [colorz](https://github.com/metakirby5/colorz) for pallete generator algorithm
- [osu](https://github.com/ppy/osu) for the idea of ​​calling cache files by their sha256 sum

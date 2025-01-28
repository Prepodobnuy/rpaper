![rpaper logo](rpaper.png)

## About
Rpaper is a program for making it easier to color windows in your cool and modern GNU/Linux distribution.

Rpaper can split wallpapers into multiple screens, set wallpapers on multiple screens, get a color palette from wallpapers, and apply a color palette to config files.

Rpaper is still **under active development**, so don't be surprised that after the update your pc will burn and configuration files will be sent to Microsoft servers.
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
- ~~get rid of pywal~~
- ~~client add arguments~~
- ~~readme~~
- ~~daemon unix socket listener~~
- daemon http adress listener
- video support using [mpvpaper](https://github.com/GhostNaN/mpvpaper)
- write own wallpaper setter (wayland-client)
- add diferent image setting modes (fit/fill/crop/etc)
- ~~implement sane --help for client~~
- implement sane --help for daemon
- implement sane logger for daemon
- write wiki
- add animated files(.gif/.png/.webp) splitting support (currently they split without animation)
- maybe abandon json format for config
- release
## Thanks to
- [colorz](https://github.com/metakirby5/colorz) for pallete generator algorithm
- [osu](https://github.com/ppy/osu) for the idea of ​​calling cache files by their sha256 sum

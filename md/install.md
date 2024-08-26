## Install
### from source
```bash
git clone https://github.com/Prepodobnuy/rpaper.git
cd rpaper
mkdir ~/.config/rpaper
cp configs/config.json ~/.config/rpaper/config.json
cp configs/color_variables.json ~/.config/rpaper/color_variables.json
cp configs/templates.json ~/.config/rpaper/templates.json
mkdir ~/.cache/rpaper
mkdir ~/.cache/rpaper/Wallpapers
cargo install --path .
chmod +X dist/rpaper_cache
sudo mv ~/.cargo/bin/rpaper /usr/bin/rpaper
mv rpaper.desktop ~/.local/share/applications/rpaper.desktop
mv rpaper.png ~/.local/share/icons/rpaper.png
cd ..
rm -rf rpaper
```

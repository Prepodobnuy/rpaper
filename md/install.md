## Install
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
python -m venv env
. env/bin/activate
pip install pyinstaller pillow
pyinstaller --clean --onefile --name rpaper_cache src/cache.py
chmod +X dist/rpaper_cache
sudo mv ~/.cargo/bin/rpaper /usr/bin/rpaper
sudo mv dist/rpaper_cache /usr/bin/rpaper_cache
cd ..
rm -rf rpaper
```
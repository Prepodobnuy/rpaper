```bash
mkdir ~/.config/rpaper
mkdir ~/.cache/rpaper
mkdir ~/.cache/rpaper/Wallpapers
cp config.json ~/~/.config/rpaper/config.json
cargo init
cargo install --path .
sudo rm /usr/bin/rpaper
sudo mv ~/.cargo/bin/rpaper /usr/bin/rpaper
python -m venv env
. env/bin/activate
pip install pyinstaller pillow
pyinstaller --clean --onefile --name rpaper_cache src/cache.py
chmod +X dist/rpaper_cache
sudo rm /usr/bin/rpaper_cache
sudo mv dist/rpaper_cache /usr/bin/rpaper_cache
```
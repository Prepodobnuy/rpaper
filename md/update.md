## Update
```bash
git clone https://github.com/Prepodobnuy/rpaper.git
cd rpaper
cargo install --path .
python -m venv env
. env/bin/activate
pip install pyinstaller pillow
pyinstaller --clean --onefile --name rpaper_cache src/cache.py
chmod +X dist/rpaper_cache
sudo rm /usr/bin/rpaper
sudo rm /usr/bin/rpaper_cache
sudo mv ~/.cargo/bin/rpaper /usr/bin/rpaper
sudo mv dist/rpaper_cache /usr/bin/rpaper_cache
cd ..
rm -rf rpaper
```
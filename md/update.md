## Update
```bash
git clone https://github.com/Prepodobnuy/rpaper.git
cd rpaper
cargo install --path .
chmod +X dist/rpaper_cache
sudo rm /usr/bin/rpaper
sudo mv ~/.cargo/bin/rpaper /usr/bin/rpaper
cd ..
rm -rf rpaper
```
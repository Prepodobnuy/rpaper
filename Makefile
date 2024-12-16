CLIENT_NAME = rpaper
DAEMON_NAME = rpaper-daemon
CLIENT_BIN_PATH = target/release/$(CLIENT_NAME)
DAEMON_BIN_PATH = target/release/$(DAEMON_NAME)
DESKTOP_FILE = rpaper.desktop
ICON = rpaper.png
RUSTC = rustc
CARGO = cargo
all: build
build:
	$(CARGO) build --release
install: build
	sudo cp $(CLIENT_BIN_PATH) /usr/bin/$(CLIENT_NAME)
	sudo cp $(DAEMON_BIN_PATH) /usr/bin/$(DAEMON_NAME)
	mkdir -p ~/.cache/rpaper
	mkdir -p ~/.cache/rpaper/wallpapers
	mkdir -p ~/.cache/rpaper/rwal
	cp $(ICON) ~/.local/share/applications/$(ICON)
	cp $(DESKTOP_FILE) ~/.local/share/applications/$(DESKTOP_FILE)
clean:
	$(CARGO) clean
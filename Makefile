CLIENT_NAME = rpaper
DAEMON_NAME = rpaper-daemon
CLIENT_BIN_PATH = target/release/$(CLIENT_NAME)
DAEMON_BIN_PATH = target/release/$(DAEMON_NAME)
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
clean:
	$(CARGO) clean
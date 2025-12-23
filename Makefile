INSTALL_PATH = ${GW2_PATH}/addons/arcdps/arcdps_chat_log.dll
TARGET = x86_64-pc-windows-gnu
WINDRES = x86_64-w64-mingw32-windres

build-debug:
	cargo build

build:
	cargo build --profile=release-with-debug

build-release:
	cargo build --release

build-windows:
	WINDRES=$(WINDRES) cargo build --target $(TARGET) --release

build-windows-debug:
	WINDRES=$(WINDRES) cargo build --target $(TARGET) --profile=release-with-debug

copy-debug:
	cp -f target/debug/arcdps_chat_log.dll "$(INSTALL_PATH)"

copy:
	cp -f target/release-with-debug/arcdps_chat_log.dll "$(INSTALL_PATH)"

copy-release:
	cp -f target/release/arcdps_chat_log.dll "$(INSTALL_PATH)"

install-debug: build-debug copy-debug

install: build copy

install-release: build-release copy-release

lint:
	cargo fmt
	cargo clippy

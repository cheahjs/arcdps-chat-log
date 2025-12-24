INSTALL_PATH = ${GW2_PATH}/addons/arcdps/arcdps_chat_log.dll
TARGET = x86_64-pc-windows-msvc
TARGET_DIR = target/$(TARGET)

build-debug:
	cargo build

build:
	cargo build --profile=release-with-debug

build-release:
	cargo build --release

build-windows:
	cargo xwin build --target $(TARGET) --release

build-windows-debug:
	cargo xwin build --target $(TARGET) --profile=release-with-debug

build-windows-release: build-windows

copy-debug:
	cp -f target/debug/arcdps_chat_log.dll "$(INSTALL_PATH)"

copy:
	cp -f target/release-with-debug/arcdps_chat_log.dll "$(INSTALL_PATH)"

copy-release:
	cp -f target/release/arcdps_chat_log.dll "$(INSTALL_PATH)"

copy-windows-release:
	cp -f $(TARGET_DIR)/release/arcdps_chat_log.dll "$(INSTALL_PATH)"

copy-windows:
	cp -f $(TARGET_DIR)/release-with-debug/arcdps_chat_log.dll "$(INSTALL_PATH)"

install-debug: build-debug copy-debug

install: build copy

install-release: build-release copy-release

install-windows: build-windows-debug copy-windows

install-windows-release: build-windows-release copy-windows-release

lint:
	cargo fmt
	cargo clippy

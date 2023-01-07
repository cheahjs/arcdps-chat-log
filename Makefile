INSTALL_PATH = ${GW2_PATH}/addons/arcdps/arcdps_chat_log.dll

build-debug:
	cargo build

build:
	cargo build --profile=release-with-debug

build-release:
	cargo build --release

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

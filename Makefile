build:
	@echo
	wasm-pack build

test:
	@echo
	cargo test

web-test:
	@echo
	wasm-pack test --headless --firefox

npm-install:
	cd www && npm install

serve: npm-install
	cd www && npm run start

watch-build:
	cargo watch -w src/ -s 'make build'

watch-test:
	cargo watch -w src/ -s 'make test'

watch-web-test:
	cargo watch -w src/ -s 'make web-test'

dev-env:
	$(MAKE) -j 3 watch-build watch-test serve

setup:
	rustup default nightly

ci:
	nix-shell shell.nix --run 'make setup test'

TARGET_FILE := pkg/rust_wasm_components_bg.wasm
release:
	wasm-pack build --release
	cp -f $(TARGET_FILE) $(TARGET_FILE).unpackaged
	wasm-opt -Oz -o $(TARGET_FILE) $(TARGET_FILE).unpackaged
	cd www && npm run build

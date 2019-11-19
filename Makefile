build:
	@echo
	wasm-pack build

npm-install:
	cd www && npm install

serve: npm-install
	cd www && npm run start

watch-build:
	cargo watch -w src/ -s 'make build'

dev-env:
	$(MAKE) -j 2 watch-build serve

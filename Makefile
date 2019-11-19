build:
	@echo
	wasm-pack build

npm-install:
	cd www && npm install

serve: npm-install
	cd www && npm run start

watch-build:
	cargo watch -w src/ -s 'make build'

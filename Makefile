.PHONY: build dev serve clean wasm components site

# Full production build
build: wasm components site

# Build WASM crate
wasm:
	wasm-pack build crates/rune-reactor-wasm --target web --release
	cp crates/rune-reactor-wasm/pkg/rune_reactor_wasm_bg.wasm site/static/
	cp crates/rune-reactor-wasm/pkg/rune_reactor_wasm.js site/static/

# Build Lit components
components:
	cd components && npm run build
	cp components/dist/rf-components.js site/static/

# Build Zola site
site:
	cd site && zola build

# Dev mode: build everything then serve
dev: wasm components
	cd site && zola serve

# Just serve (assumes assets already built)
serve:
	cd site && zola serve

# Run all Rust tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean
	rm -rf site/public
	rm -rf components/dist
	rm -f site/static/*.wasm site/static/*.js
	rm -rf crates/rune-reactor-wasm/pkg

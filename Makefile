.PHONY: test test-native test-wasm coverage

test: test-native test-wasm

test-native:
	cargo test --workspace --exclude pt-wasm

test-wasm:
	wasm-pack test --node crates/pt-wasm

coverage:
	cargo llvm-cov --workspace --exclude pt-wasm --features pt-data/bundled --lcov --output-path lcov.info
	wasm-pack test --node crates/pt-wasm

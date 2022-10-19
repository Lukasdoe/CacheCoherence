RUST_BACKTRACE := 0

run:
	cargo run -- dragon ./data/01.zip 128 2 8 --no-progress

build:
	cargo build

.PHONY: run build

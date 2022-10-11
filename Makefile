RUST_BACKTRACE := 0

cli:
	cargo run -- dragon ./data/01.zip 128 2 8

build:
	cargo build

.PHONY: cli

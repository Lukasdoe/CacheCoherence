RUST_BACKTRACE := 0

run:
	cargo run -- dragon ./data/blackscholes/blackscholes_10.zip 128 2 8 --no-progress

build:
	cargo build

.PHONY: run build

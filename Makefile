cli:
	cargo run -- dragon ./data/01.zip 128 2 8

build:
	cargo build

gui:
	cd frontend && cargo run &
	cd frontend && npm run --prefix gui dev

build-gui:
	cd frontend && \
	npm --prefix gui ci && \
	cargo build

.PHONY: cli gui

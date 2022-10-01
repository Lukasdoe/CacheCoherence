cli:
	cargo run mesi data/02 256 1 8

gui:
	npm run --prefix gui dev &
	cargo run --bin coherence-gui

.PHONY: cli gui


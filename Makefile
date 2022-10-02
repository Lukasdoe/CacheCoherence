cli:
	cargo run mesi data/02 256 1 8

gui:
	cargo run --bin coherence-gui &
	npm run --prefix gui dev

.PHONY: cli gui


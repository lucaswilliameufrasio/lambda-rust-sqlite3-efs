init:
	cargo install cargo-binstall
	cargo binstall cargo-watch
	cargo check
PHONY: init


watch:
	cargo watch -x 'run'
PHONY: watch

init:
	cargo install cargo-binstall
	cargo binstall cargo-watch
	cargo install --path .
PHONY: init

watch:
	cargo watch -x 'run'
PHONY: watch

dev:
	docker run --rm --interactive --tty \
        --workdir /app \
        --volume ${CARGO_HOME:-$$HOME/.cargo}:/.cargo \
        --volume $$PWD:/app \
        --env CARGO_HOME=/.cargo \
		--env PORT=9000 \
  		-p 9000:9000 \
  		calavera/cargo-lambda cargo lambda watch
.PHONY: dev

build:
	docker run --rm --interactive --tty \
        --workdir /app \
        --volume $${CARGO_HOME:-$$HOME/.cargo}:/.cargo \
        --volume $$PWD:/app \
        --env CARGO_HOME=/.cargo \
  		calavera/cargo-lambda cargo lambda build --release $(CARGO_LAMBDA_FLAGS)
	cp  ./target/lambda/rust-axum-lambda/bootstrap bootstrap
.PHONY: build

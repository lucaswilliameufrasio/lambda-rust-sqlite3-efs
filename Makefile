init:
	cargo install cargo-binstall
	cargo binstall cargo-watch
	cargo install sqlx-cli --no-default-features --features native-tls,sqlite
	cargo install cargo-edit
	cargo install cargo-upgrades
	sqlx db create
	cargo install --path .

	@if which asdf >/dev/null 2>&1; then \
		asdf reshim rust; \
	fi
	
	sqlx migrate run
	cargo sqlx prepare
PHONY: init

set-env:
	export DATABASE_URL="sqlite:users.db"
PHONY: set-env

dev:
	cargo watch -x 'run'
.PHONY: dev

build:
	cargo lambda build --release --arm64
.PHONY: build

prepare-deploy:
	$(MAKE) build
	@ cp ./target/lambda/api/bootstrap bootstrap-api
	@ cp ./target/lambda/writer/bootstrap bootstrap-writer
	@ rm -f bootstrap-api.zip bootstrap-writer.zip
	@ zip bootstrap-api.zip ./bootstrap-api
	@ zip bootstrap-writer.zip ./bootstrap-writer
	@ rm -f bootstrap-api bootstrap-writer
.PHONY: prepare-deploy

deploy:
	$(MAKE) prepare-deploy
	@ ./scripts/deploy-functions.sh
.PHONY: deploy

smoke-test:
	cargo test --lib
.PHONY: smoke-test

floci-test:
	@bash scripts/smoke-test.sh
.PHONY: floci-test

fmt:
	cargo fmt --all -- --check
.PHONY: fmt

lint:
	cargo clippy --all-targets --all-features -- -D warnings
.PHONY: lint

check: fmt lint test
.PHONY: check

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
	make build
	@ cp ./target/lambda/lambda-rust-sqlite3-efs/bootstrap bootstrap
	@ rm bootstrap.zip && zip bootstrap ./bootstrap
.PHONY: prepare-deploy

deploy:
	make prepare-deploy
	@ ./scripts/deploy-functions.sh
.PHONY: deploy
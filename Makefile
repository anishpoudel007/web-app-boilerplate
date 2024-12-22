.PHONY: dev

dev:
	cargo watch -qcx run

release:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy --all-targets --all-features

entity:
	sea-orm-cli generate entity -o src/models/_entities --with-serde serialize

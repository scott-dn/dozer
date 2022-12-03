init:
	cargo check

build:
	cargo build --release
	docker build -t data-processor .

local:
	cargo run --release

format:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings -D clippy::all

clean:
	cargo clean

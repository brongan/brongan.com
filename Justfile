# print recipes
_default:
  just --list

# Run linters
format:
	cargo fmt
	cargo clippy --fix --allow-dirty

# Emulate cargo leptos build
build:
	cargo build --package=frontend --lib --target-dir=/home/brong/ws/brongan.com/target/front --target=wasm32-unknown-unknown --no-default-features
	cargo build --package=server --bin=server --no-default-features

# Emulate cargo leptos serve
run: build
	LEPTOS_OUTPUT_NAME=brongan_com ./target/debug/server

# hotreload backend and frontend
develop:
	cargo leptos watch

# run server with nix (local filesystem and no tls)
nix-run:
	nix run

# nix build and pipe into podman
container:
	nix build .#docker
	./result | podman load

# run podman container (docker filesystem)
container-run: container
	podman run -e SENTRY_DSN -e HONEYCOMB_API_KEY -p 8080:8080 localhost/brongan_com:latest

# This should succeed before commiting.
precommit: format build container 

# push to fly.io
deploy: container
	podman push brongan_com docker://registry.fly.io/rust-brongan-com:latest
	flyctl deploy -i registry.fly.io/rust-brongan-com:latest



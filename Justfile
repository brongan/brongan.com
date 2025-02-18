# print recipes
_default:
  just --list

# cargo build --all
build:
	cargo build --all

# nix build and pipe into podman
container:
	nix build .#dockerImage
	./result | podman load

# run podman container (docker filesystem)
container-run: container
	podman run -e SENTRY_DSN -e HONEYCOMB_API_KEY -p 8080:8080 localhost/brongan_com:latest

# run server with nix (local filesystem and no tls)
run:
	nix run

# run server with cargo
local-run: build
	cargo leptos serve

# linters!
format:
	cargo fmt
	cargo clippy --fix --allow-dirty

# This should succeed before commiting.
precommit: format build container 

# push to fly.io
deploy: container
	podman push brongan_com docker://registry.fly.io/rust-brongan-com:latest
	flyctl deploy -i registry.fly.io/rust-brongan-com:latest

# hotreload backend and frontend
develop:
	cargo leptos watch


_default:
  just --list

build:
	cargo build --target=wasm32-unknown-unknown --no-default-features --features hydrate
	cargo build --target x86_64-unknown-linux-musl --no-default-features --features ssr

trunk:
	trunk build

container:
	nix build .#dockerImage
	./result | podman load

run:
	cargo leptos serve

develop:
	cargo leptos watch

deploy: container
  podman push brongan_com docker://registry.fly.io/rust-brongan-com:latest
  flyctl deploy -i registry.fly.io/rust-brongan-com:latest


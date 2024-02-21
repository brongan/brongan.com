_default:
  just --list

build:
	cargo leptos build

container:
	nix build .#dockerImage
	./result | podman load

run: build
	cargo leptos serve

develop: build
	cargo leptos watch

deploy: container
  podman push brongan_com docker://registry.fly.io/rust-brongan-com:latest
  flyctl deploy -i registry.fly.io/rust-brongan-com:latest


_default:
  just --list

build:
	cargo build --all

container:
	nix build .#dockerImage
	./result | podman load

container-run: container
	nix run

run: build
	cargo run --bin server -- --port 8081 --ssl-port 8443 --static-dir client/dist --cert-dir cert --dev

deploy: container
  podman push brongan_com docker://registry.fly.io/rust-brongan-com:latest
  flyctl deploy -i registry.fly.io/rust-brongan-com:latest

opc: container
  podman save localhost/brongan-com | ssh opc podman load

develop:
  ./develop.sh


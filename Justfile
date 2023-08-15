_default:
  just --list

build:
	nix build .#dockerImage
	./result | podman load

run:
	nix run

deploy:
  just build
  podman push brongan_com docker://registry.fly.io/rust-brongan-com:latest
  flyctl deploy -i registry.fly.io/rust-brongan-com:latest

opc:
  just build
  podman save localhost/brongan-com | ssh opc podman load

develop:
  ./develop.sh

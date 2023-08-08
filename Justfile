_default:
  just --list

build:
	nix build .#dockerImage
	./result | docker load

run:
	nix run

deploy:
  just build
  podman save localhost/brongan-com | ssh opc podman load

develop:
  ./develop.sh

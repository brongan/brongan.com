_default:
  just --list

build:
	nix build .#dockerImage
	./result | docker load

run:
  nix run

deploy:
  just build
  podman push catscii docker://registry.fly.io/late-wood-6224:latest
  flyctl deploy -i registry.fly.io/late-wood-6224:latest


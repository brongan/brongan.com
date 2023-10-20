_default:
  just --list

build:
	env RUSTFLAGS="-C target-feature=+crt-static" cargo build --target=x86_64-unknown-linux-musl --bin server 
	cargo build --target=wasm32-unknown-unknown --bin client

container:
	nix build .#dockerImage
	./result | podman load

run:
	env RUSTFLAGS="-C target-feature=+crt-static" cargo run --target=x86_64-unknown-linux-musl --bin server -- --port 8081 --ssl-port 8443  --static-dir client/dist --cert-dir cert --dev

deploy:
  just container
  podman push brongan_com docker://registry.fly.io/rust-brongan-com:latest
  flyctl deploy -i registry.fly.io/rust-brongan-com:latest

opc:
  just build
  podman save localhost/brongan-com | ssh opc podman load

develop:
  ./develop.sh

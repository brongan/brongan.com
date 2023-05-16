FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
	cargo build --release && mv ./target/release/catscii ./catscii

# Runtime image
FROM debian:bullseye-slim

RUN set -eux; \
		apt update; \
		apt install -y --no-install-recommends \
			ca-certificates \
			; \
		apt clean autoclean; \
		apt autoremove --yes; \
		rm -rf /var/lib/{apt,dpkg,cache,log}/

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/app/catscii /app/catscii

EXPOSE 8080

# Run the app
CMD ./catscii

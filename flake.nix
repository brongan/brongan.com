{
  description = "My personal website!.";
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
        flake-utils.follows = "flake-utils";
      };
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay  }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        overlays = [ (import rust-overlay) ];
		rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        commonArgs = {
          inherit src;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        bin = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        });
        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "brongan_com";
          tag = "latest";
          contents = [ bin ];
          config = {
            Cmd = [ "${bin}/bin/server" ];
            Env = with pkgs; [ "GEOLITE2_COUNTRY_DB=${clash-geoip}/etc/clash/Country.mmdb" ];
          };
        };
      in
      with pkgs;
      {
        packages =
          {
            inherit bin dockerImage;
            default = bin;
          };
      }
    );
}

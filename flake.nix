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
  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ...  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system ;
          config.allowUnfree = true;
		  overlays = [ (import rust-overlay) ];
        };
		inherit (pkgs) lib;
		rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
		src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.scss" path) ||
            (lib.hasSuffix "\.frag" path) ||
            (lib.hasSuffix "\.vert" path) ||
			(lib.hasInfix "/img/" path) ||
			(lib.hasInfix "/resources/" path) ||
            (craneLib.filterCargoSources path type)
          ;
        };
        commonArgs = {
          inherit src;
		  pname = "brongan.com";
          version = "0.1.0";
        };
		nativeArgs = commonArgs // {
			pname = "trunk-workspace-native";
		};
        cargoArtifacts = craneLib.buildDepsOnly nativeArgs;
        server = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
		  CLIENT_DIST = myClient;
        });
		wasmArgs = commonArgs // {
          pname = "trunk-workspace-wasm";
          cargoExtraArgs = "--package=root --package=wasm-game-of-life --package=ishihara";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        };
		cargoArtifactsWasm = craneLib.buildDepsOnly (wasmArgs // {
          doCheck = false;
        });
		myClient = craneLib.buildTrunkPackage (wasmArgs // {
          pname = "trunk-workspace-client";
          cargoArtifacts = cargoArtifactsWasm;
          trunkIndexPath = "root/index.html";
        });
        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "brongan_com";
          tag = "latest";
          contents = [ server ];
          config = {
            Cmd = [ "${server}/bin/server" ];
            Env = with pkgs; [ "GEOLITE2_COUNTRY_DB=${clash-geoip}/etc/clash/Country.mmdb" ];
          };
        };
      in
      with pkgs;
      {
        packages =
          {
            inherit server dockerImage;
            default = server;
          };
      }
    );
}

{
  description = "My personal website!.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs-for-wasm-bindgen.url = "github:NixOS/nixpkgs/4e6868b1aa3766ab1de169922bb3826143941973";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, nixpkgs-for-wasm-bindgen, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          overlays = [ (import rust-overlay) ];
        };
        sqliteStatic = pkgs.pkgsMusl.sqlite.override {
          stdenv =
            pkgs.pkgsStatic.stdenv;
        };
        inherit (pkgs) lib;
        wasmToolchain = pkgs.rust-bin.nightly.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        nativeToolchain = pkgs.rust-bin.nightly.latest.default.override {
          targets = [ "x86_64-unknown-linux-musl" ];
        };
        wasmCraneLib = ((crane.mkLib pkgs).overrideToolchain wasmToolchain).overrideScope (final: prev: {
          inherit (import nixpkgs-for-wasm-bindgen { inherit system; }) wasm-bindgen-cli;
        });
        nativeCraneLib = (crane.mkLib pkgs).overrideToolchain nativeToolchain;
		css = pkgs.stdenv.mkDerivation {
			name = "css";
			src = self;
			nativeBuildInputs = with pkgs; [ dart-sass tree ];
			buildPhase = "sass ./style/main.scss main.css";
			installPhase = "mkdir -p $out; install -t $out main.css";
		};
        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.frag" path) ||
            (lib.hasSuffix "\.vert" path) ||
            (lib.hasInfix "/assets/" path) ||
            (wasmCraneLib.filterCargoSources path type)
          ;
        } // css;
        commonArgs = {
          inherit src;
          pname = "brongan.com";
          version = "0.1.0";
          strictDeps = true;
          nativeBuildInputs = with pkgs; [ pkg-config ];
        };
        nativeArgs = commonArgs // {
          cargoExtraArgs = "--no-default-features --features=ssr";
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
          buildInputs = [ sqliteStatic ];
        };
        nativeArtifacts = nativeCraneLib.buildDepsOnly nativeArgs;
        myServer = nativeCraneLib.buildPackage (nativeArgs // {
		  pname = "brongan-com-server";
          cargoArtifacts = nativeArtifacts;
          CLIENT_DIST = myClient;
        });
        wasmArgs = commonArgs // {
          cargoExtraArgs = "--no-default-features --features=hydrate";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          doCheck = false;
        };
        wasmArtifacts = wasmCraneLib.buildDepsOnly wasmArgs;
        myClient = wasmCraneLib.buildTrunkPackage (wasmArgs // {
          pname = "brongan-com-client";
          cargoArtifacts = wasmArtifacts;
		  trunkIndexPath = "./index.html";
		  trunkExtraBuildArgs = "--no-default-features --features hydrate";
		  wasm-bindgen-cli = pkgs.wasm-bindgen-cli.override {
            version = "0.2.90";
            hash = "sha256-X8+DVX7dmKh7BgXqP7Fp0smhup5OO8eWEhn26ODYbkQ=";
            cargoHash = "sha256-ckJxAR20GuVGstzXzIj1M0WBFj5eJjrO2/DRMUK5dwM=";
          };
        });
        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "brongan_com";
          tag = "latest";
          contents = [ myServer myClient ];
          config = {
            Cmd = [
              "${myServer}/bin/server"
              "--prod"
            ];
            Env = with pkgs; [
			"GEOLITE2_COUNTRY_DB=${clash-geoip}/etc/clash/Country.mmdb"
			"RUST_LOG=info"
			"LEPTOS_OUTPUT_NAME=brongan"
			"LEPTOS_SITE_ADDR=0.0.0.0:8080"
			"LEPTOS_SITE_ROOT=site"
			];
          };
        };
      in
      {
        packages = {
          inherit myServer dockerImage;
          default = myServer;
        };
      }
    );
}

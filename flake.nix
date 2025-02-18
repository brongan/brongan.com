{
  description = "My personal website!.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs-for-wasm-bindgen.url = "github:NixOS/nixpkgs/4e6868b1aa3766ab1de169922bb3826143941973";
    crane = {
      url = "github:ipetkov/crane";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = { nixpkgs, crane, flake-utils, rust-overlay, nixpkgs-for-wasm-bindgen, ... }:
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
        wasmCraneLib = ((crane.mkLib pkgs).overrideToolchain wasmToolchain);
        nativeCraneLib = (crane.mkLib pkgs).overrideToolchain nativeToolchain;
        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.scss" path) ||
            (lib.hasSuffix "\.frag" path) ||
            (lib.hasSuffix "\.vert" path) ||
            (lib.hasSuffix "\.ttf" path) ||
            (lib.hasInfix "/img/" path) ||
            (lib.hasInfix "/resources/" path) ||
            (wasmCraneLib.filterCargoSources path type)
          ;
        };
        commonArgs = {
          inherit src;
          pname = "brongan.com";
          version = "0.1.0";
          strictDeps = true;
          nativeBuildInputs = with pkgs; [ pkg-config ];
        };
        nativeArgs = commonArgs // {
          pname = "server";
          cargoExtraArgs = "--package=server --no-default-features";
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
          buildInputs = [ sqliteStatic ];
        };
        cargoArtifacts = nativeCraneLib.buildDepsOnly nativeArgs;
        myServer = nativeCraneLib.buildPackage (nativeArgs // {
          inherit cargoArtifacts;
          CLIENT_DIST = myClient;
        });
        wasmArgs = commonArgs // {
          pname = "frontend";
          cargoExtraArgs = "--package=frontend --no-default-features";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        };
        cargoArtifactsWasm = wasmCraneLib.buildDepsOnly (wasmArgs // {
          doCheck = false;
        });
        myClient = wasmCraneLib.buildPackage (wasmArgs // {
          pname = "brongan-com-frontend";
          doCheck = false;
          cargoArtifacts = cargoArtifactsWasm;
          inherit (import nixpkgs-for-wasm-bindgen { inherit system; }) wasm-bindgen-cli;
        });
        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "brongan_com";
          tag = "latest";
          contents = [ myServer myClient ];
          config = {
            Cmd = [ "${myServer}/bin/server" ];
            Env = with pkgs; [
				"GEOLITE2_COUNTRY_DB=${dbip-country-lite}/share/dbip/dbip-country-lite.mmdb"
				"LEPTOS_SITE_ADDR=0.0.0.0:8080"
				"LEPTOS_ENV=PROD"
				"LEPTOS_OUTPUT_NAME=brongan_com"
				"DB=sqlite.db"
			];
            WorkingDir = "/";
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

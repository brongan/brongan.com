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
  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs) lib;
        wasmToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        nativeToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "x86_64-unknown-linux-musl" ];
        };
        wasmCraneLib = (crane.mkLib pkgs).overrideToolchain wasmToolchain;
        nativeCraneLib = (crane.mkLib pkgs).overrideToolchain nativeToolchain;
        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.scss" path) ||
            (lib.hasSuffix "\.frag" path) ||
            (lib.hasSuffix "\.vert" path) ||
            (lib.hasInfix "/img/" path) ||
            (lib.hasInfix "/resources/" path) ||
            (wasmCraneLib.filterCargoSources path type)
          ;
        };
        commonArgs = {
          inherit src;
          pname = "brongan.com";
          version = "0.1.0";
        };
        nativeArgs = commonArgs // {
          pname = "server";
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
        };
        cargoArtifacts = nativeCraneLib.buildDepsOnly nativeArgs;
        myServer = nativeCraneLib.buildPackage (nativeArgs // {
          inherit cargoArtifacts;
          CLIENT_DIST = myClient;
        });
        wasmArgs = commonArgs // {
          pname = "client";
          cargoExtraArgs = "--package=client";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        };
        cargoArtifactsWasm = wasmCraneLib.buildDepsOnly (wasmArgs // {
          doCheck = false;
        });
        myClient = wasmCraneLib.buildTrunkPackage (wasmArgs // {
          pname = "brongan-com-client";
          cargoArtifacts = cargoArtifactsWasm;
          trunkIndexPath = "client/index.html";
        });
        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "brongan_com";
          tag = "latest";
          contents = [ myServer myClient ];
          config = {
            Cmd = [ "${myServer}/bin/server" "--static-dir=\"\"" ];
            Env = with pkgs; [ "GEOLITE2_COUNTRY_DB=${clash-geoip}/etc/clash/Country.mmdb" ];
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

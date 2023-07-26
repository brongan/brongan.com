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
  outputs = { nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (localSystem:
      let
        crossSystem = "aarch64-linux";
        pkgs = import nixpkgs {
          inherit crossSystem localSystem;
          config.allowUnfree = true;
          overlays = [ (import rust-overlay) ];
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
          targets = [ "aarch64-unknown-linux-gnu" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        inherit (pkgs) lib;
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
        crateExpression =
          { sqlite, libiconv, lib, pkg-config, qemu, stdenv }:
          craneLib.buildPackage {
            inherit src;
            depsBuildBuild = [ qemu ];
            nativeBuildInputs = [ pkg-config ];
            buildInputs = [ sqlite ];
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-aarch64";
            cargoExtraArgs = "--target aarch64-unknown-linux-gnu";
            HOST_CC = "${stdenv.cc.nativePrefix}cc";

            pname = "server";
            version = "0.1.0";
            CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";
            # CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
            CLIENT_DIST = myClient;
          };
        myServer = pkgs.callPackage crateExpression { };
        wasmToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        wasmCraneLib = (crane.mkLib pkgs).overrideToolchain wasmToolchain;
        wasmArgs = {
          inherit src;
          version = "0.1.0";

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
          name = "brongan-com";
          tag = "latest";
          contents = [ myServer myClient ];
          config = {
            Cmd = [ "${myServer}/bin/server" ];
            Env = with pkgs; [ "GEOLITE2_COUNTRY_DB=${clash-geoip}/etc/clash/Country.mmdb" ];
          };
        };
      in
      {
        checks = {
          inherit myServer;
        };
        packages = {
          inherit myServer dockerImage;
          default = myServer;
        };
        apps.default = flake-utils.lib.mkApp {
          drv = pkgs.writeScriptBin "my-app" ''
            ${pkgs.pkgsBuildBuild.qemu}/bin/qemu-aarch64 ${myServer}/bin/server
          '';
        };
      });
}

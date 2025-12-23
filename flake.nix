{
  description = "My personal website!";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        crossPkgs = import nixpkgs {
          inherit system;
          crossSystem = "x86_64-unknown-linux-musl";
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        baseCraneLib = crane.mkLib pkgs;

        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            (baseCraneLib.fileset.commonCargoSources ./.)
            (lib.fileset.fileFilter (
              file:
              lib.any file.hasExt [
                "html"
                "scss"
                "css"
                "json"
                "ttf"
                "frag"
                "vert"
              ]
            ) ./.)
          ];
        };

        commonArgs = {
          inherit src;
          pname = "brongan.com";
          version = "0.1.0";
          strictDeps = true;
        };

        wasmCraneLib = baseCraneLib.overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          }
        );

        wasmBindgenCli = pkgs.buildWasmBindgenCli rec {
          src = pkgs.fetchCrate {
            pname = "wasm-bindgen-cli";
            version = "0.2.106";
            hash = "sha256-zLPFFgnqAWq5R2KkaTGAYqVQswfBEYm9x3OPjx8DJRY=";
          };
          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            inherit src;
            inherit (src) pname version;
            hash = "sha256-a2X9bzwnMWNt0fTf30qAiJ4noal/ET1jEtf5fBFj5OU=";
          };
        };

        wasmArgs = commonArgs // {
          pname = "brongan-client";
          cargoExtraArgs = "--package=frontend --no-default-features";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          doCheck = false;
          nativeBuildInputs = [ pkgs.pkg-config ];
        };

        cargoArtifactsWasm = wasmCraneLib.buildDepsOnly wasmArgs;

        myClient = wasmCraneLib.buildTrunkPackage (
          wasmArgs
          // {
            pname = "brongan-client";
            cargoArtifacts = cargoArtifactsWasm;
            wasm-bindgen-cli = wasmBindgenCli;
            preBuild = "cd frontend";
            postBuild = ''
              cd dist
              mkdir -p site/pkg
              HASHED_JS=$(find . -maxdepth 1 -name "*.js" | head -n 1)
              HASHED_WASM=$(find . -maxdepth 1 -name "*.wasm" | head -n 1)
              HASHED_CSS=$(find . -maxdepth 1 -name "*.css" | head -n 1)
              CLEAN_WASM_NAME=''${HASHED_WASM#./}
              sed -i "s|$CLEAN_WASM_NAME|frontend_bg.wasm|g" "$HASHED_JS"
              mv "$HASHED_JS" site/pkg/frontend.js
              mv "$HASHED_WASM" site/pkg/frontend_bg.wasm
              mv "$HASHED_CSS" site/pkg/brongan_com.css
              mv index.html site/pkg
              cd ../..
              mv frontend/dist dist
            '';
          }
        );

        crossCraneLib = (crane.mkLib crossPkgs).overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            targets = [ "x86_64-unknown-linux-musl" ];
          }
        );

        nativeArgs = commonArgs // {
          pname = "server";
          cargoExtraArgs = "--package=server --no-default-features";
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";

          nativeBuildInputs = [
            crossPkgs.buildPackages.pkg-config
            crossPkgs.buildPackages.cmake
          ];

          buildInputs = [ crossPkgs.sqlite ];
        };

        cargoArtifactsServer = crossCraneLib.buildDepsOnly nativeArgs;

        myServer = crossCraneLib.buildPackage (
          nativeArgs
          // {
            cargoArtifacts = cargoArtifactsServer;
            CLIENT_DIST = myClient;
          }
        );

        myAssets =
          pkgs.runCommand "brongan-assets"
            {
              src = ./public;
            }
            ''
              mkdir -p $out/site
              cp -r $src/* $out/site/
            '';

        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "brongan_com";
          tag = "latest";
          contents = [
            myServer
            myClient
            myAssets
          ];
          config = {
            Cmd = [ "${myServer}/bin/server" ];
            Env = [
              "GEOLITE2_COUNTRY_DB=${pkgs.dbip-country-lite}/share/dbip/dbip-country-lite.mmdb"
              "LEPTOS_SITE_ADDR=0.0.0.0:8080"
              "LEPTOS_SITE_ROOT=/site"
              "LEPTOS_SITE_PACKAGE_DIR=/pkg/"
              "LEPTOS_ENV=PROD"
              "LEPTOS_OUTPUT_NAME=frontend"
              "DB=sqlite.db"
            ];
            WorkingDir = "/";
            ExposedPorts = {
              "8080/tcp" = { };
            };
          };
        };
      in
      {
        packages = {
          default = myServer;
          client = myClient;
          docker = dockerImage;
        };

        devShells.default = wasmCraneLib.devShell {
          checks = self.packages.${system};
          packages = with pkgs; [
            trunk
            wasmBindgenCli
            bacon
            sqlite
          ];
          LEPTOS_SITE_ROOT = "./frontend/dist";
        };
      }
    );
}

{
  description = "My personal website!.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
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
  outputs = { nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs) lib;
		cross = import nixpkgs {
			localSystem = "x86_64-linux";
			crossSystem = "x86_64-unknown-linux-musl";
			config.allowUnfree = true;
			overlays = [ (import rust-overlay) ];
		};
		wasmCraneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          }
        );
		nativeCraneLib = (crane.mkLib cross).overrideToolchain (
			p:
			p.rust-bin.stable.latest.default.override {
				targets = [ "x86_64-unknown-linux-musl" ];
			}
		);
		unfilteredRoot = ./.;
		src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (wasmCraneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.fileFilter (
              file:
              lib.any file.hasExt [
                "html"
                "scss"
				"css"
				"frag"
				"vert"
				"ttf"
				"ico"
				"png"
				"jpg"
              ]
            ) unfilteredRoot)
            (lib.fileset.maybeMissing ./image)
            (lib.fileset.maybeMissing ./resources)
          ];
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
          buildInputs = [ pkgs.pkgsStatic.sqlite ];
        };
        cargoArtifacts = nativeCraneLib.buildDepsOnly nativeArgs;
        myServer = nativeCraneLib.buildPackage (nativeArgs // {
          inherit cargoArtifacts;
          CLIENT_DIST = myClient;
        });
        wasmArgs = commonArgs // {
          pname = "brongan_com";
          cargoExtraArgs = "--package=frontend --no-default-features";
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
        };
        cargoArtifactsWasm = wasmCraneLib.buildDepsOnly (wasmArgs // {
          doCheck = false;
        });
        myClient = wasmCraneLib.buildTrunkPackage (wasmArgs // {
          pname = "brongan-com-frontend";
          doCheck = false;
          cargoArtifacts = cargoArtifactsWasm;
			preBuild = ''
              cd ./frontend
            '';
		  postBuild = ''
			mkdir -p $out/dist
			if [ -d "frontend/public" ]; then
			  cp -r frontend/public/. $out/dist/
			fi
		  '';
		   wasm-bindgen-cli = pkgs.buildWasmBindgenCli rec {
              src = pkgs.fetchCrate {
                pname = "wasm-bindgen-cli";
                version = "0.2.104";
                hash = "sha256-9kW+a7IreBcZ3dlUdsXjTKnclVW1C1TocYfY8gUgewE=";
                # hash = lib.fakeHash;
              };

              cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
                inherit src;
                inherit (src) pname version;
                hash = "sha256-V0AV5jkve37a5B/UvJ9B3kwOW72vWblST8Zxs8oDctE=";
                # hash = lib.fakeHash;
              };
            };
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
				"LEPTOS_SITE_ROOT=/dist/"
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

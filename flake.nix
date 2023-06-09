{
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
  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
			config.allowUnfree = true;
          };
		  craneLib = (crane.mkLib pkgs);
          src = craneLib.cleanCargoSource ./.;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl sqlite ];
          commonArgs = {
            inherit src buildInputs nativeBuildInputs;
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          bin = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
		  dockerImage = pkgs.dockerTools.streamLayeredImage {
			  name = "catscii";
			  tag = "latest";
			  contents = [ bin pkgs.cacert ];
			  config = {
				  Cmd = [ "${bin}/bin/catscii" ];
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
		  devShells.default = mkShell {
            inputsFrom = [ bin ];
          };
        }
      );
}

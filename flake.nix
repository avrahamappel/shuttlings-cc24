{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    self,
    flake-utils,
    naersk,
    nixpkgs,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [fenix.overlays.default];
        };

        naersk' = pkgs.callPackage naersk {};

        shuttle = naersk'.buildPackage rec {
          name = "shuttle";
          version = "0.49.0";
          src = pkgs.fetchFromGitHub {
            owner = "shuttle-hq";
            repo = "shuttle";
            rev = "v${version}";
            hash = "sha256-97AiRgTPzkNsnxMTF0zleHVR6QYLyRlhguh2nz+duUM=";
          };
        };

        cch24-validator = naersk'.buildPackage rec {
          name = "cch24-validator";
          version = "19.0.0";
          src = pkgs.fetchzip {
            url = "https://crates.io/api/v1/crates/${name}/${version}/download";
            hash = "sha256-61pQ8NFOzXhz1MBx/SBlEk6OgtwA5QAU1G4rwqvHpws=";
            extension = "tar";
          };
        };

      in {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };

        packages = {
          inherit shuttle cch24-validator;
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            alejandra
            rust-analyzer
            cargo-watch
            shuttle
            cch24-validator
            (pkgs.fenix.stable.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])

            cmake # required by boring -> jwt-simple
          ];
        };
      }
    );
}

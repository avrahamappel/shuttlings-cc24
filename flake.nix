{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";
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
          pname = "shuttle";
          version = "0.49.0";
          src = pkgs.fetchFromGitHub {
            owner = "shuttle-hq";
            repo = "shuttle";
            rev = "v${version}";
            hash = "sha256-97AiRgTPzkNsnxMTF0zleHVR6QYLyRlhguh2nz+duUM=";
          };
        };

        cch24-validator = naersk'.buildPackage rec {
          pname = "cch24-validator";
          version = "2.0.1";
          src = pkgs.fetchzip {
            url = "https://crates.io/api/v1/crates/${pname}/${version}/download";
            hash = "sha256-AdKFVGvRe3xG3bzaaAJ95NSWjJ4oCgmB8Y7UcaDVOiM=";
            extension = "tar";
          };
        };

      in rec {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            alejandra
            rust-analyzer
            shuttle
            cch24-validator
            (pkgs.fenix.stable.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
          ];
        };
      }
    );
}

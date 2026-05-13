{
  description = "Nexus semantic content vocabulary over NOTA syntax.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-gh/xTkxKHL4eiRXzWv8KP7vfjSk61Iq48x47BEDFgfk=";
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        src = craneLib.cleanCargoSource ./.;
        commonArgs = {
          inherit src;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in
      {
        packages.default = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );

        checks = {
          default = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
          nexus-uses-current-actor-runtime = pkgs.runCommand "nexus-uses-current-actor-runtime" { } ''
            retired_actor_runtime="$(printf '%s%s' rac tor)"
            ${pkgs.gnugrep}/bin/grep -R -Fq 'kameo' ${./Cargo.toml} ${./src}
            ! ${pkgs.gnugrep}/bin/grep -R -E "(^|[^[:alnum:]_])$retired_actor_runtime([^[:alnum:]_]|$)" ${./Cargo.toml} ${./src}
            touch $out
          '';
        };

        devShells.default = pkgs.mkShell {
          name = "nexus";
          packages = [
            pkgs.jujutsu
            toolchain
          ];
        };
      }
    );
}

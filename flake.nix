{
  description = "croshet development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      crane,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachSystem
      [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-arm"
        "x86_64-windows"
        "aarch64-windows"
      ]
      (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          # --- Libraries ---
          lib = pkgs.lib;
          craneLib = (crane.mkLib pkgs).overrideToolchain (
            toolchain:
            toolchain.rust-bin.stable."1.89.0".default.override {
              extensions = [
                "clippy"
                "rust-analyzer"
                "cargo"
                "rustc"
                "rust-src"
              ];
            }
          );

          # Base arguments passed to almost all crane invocations
          commonCraneArgs = {
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions ([
                (craneLib.fileset.commonCargoSources ./.)
                (lib.fileset.fromSource ./README.md)
              ]);
            };
            nativeCheckInputs = with pkgs; [
              lune
              lua
            ];
            strictDeps = true;
          };

          # Build dependencies separately to have them cached in nix store
          cargoArtifacts = craneLib.buildDepsOnly commonCraneArgs;

          # Finally, compile the actual project
          crate =
            {
              features ? [ ],
            }:
            craneLib.buildPackage (
              commonCraneArgs
              // {
                inherit cargoArtifacts;
                doChecks = false;
              }
            );

          croshet = pkgs.callPackage crate { };
        in
        {
          apps.default = flake-utils.lib.mkApp { drv = croshet; };
          packages = {
            default = croshet;
            inherit croshet;
          };

          checks = {
            inherit croshet;
            formatting = pkgs.runCommandLocal "treefmt-check" { buildInputs = [ pkgs.nixfmt-tree ]; } ''
              set -euo pipefail
              cp -r ${./.} workdir
              chmod -R +w workdir/
              treefmt --ci --tree-root workdir/
              touch $out
            '';
          };

          formatter = pkgs.nixfmt-tree;
          devShells.default = craneLib.devShell {
            inputsFrom = [ croshet ];
            checks = self.checks.${system};
            packages = with pkgs; [
              git
              nixfmt-tree
              lua
              lune
            ];

            shellHook = ''
              # Use host's default shell to make it more homely
              if [[ $- == *i* ]]; then
                export SHELL=$(getent passwd $USER | cut -d: -f7)
                exec $SHELL
              fi
            '';
          };
        }
      );
}

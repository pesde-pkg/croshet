{
  description = "deno_task_shell development environment";

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
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
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
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
          # nativeBuildInputs = with pkgs; [
          #   pkg-config
          # ];
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

        deno_task_shell = pkgs.callPackage crate { };
      in
      {
        apps.default = flake-utils.lib.mkApp { drv = deno_task_shell; };
        packages = {
          default = deno_task_shell;
          inherit deno_task_shell;
        };

        checks = {
          inherit deno_task_shell;
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
          inputsFrom = [ deno_task_shell ];
          checks = self.checks.${system};
          packages = with pkgs; [
            git
            nixfmt-tree
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

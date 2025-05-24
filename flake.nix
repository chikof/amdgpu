{
  description = "A Nix-flake-based Rust development environment";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, fenix, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        crane = inputs.crane.mkLib pkgs;

        toolchain = with fenix.packages.${system};
          combine [
            stable.rustc
            stable.rust-src
            stable.cargo
            complete.rustfmt
            stable.clippy
            stable.rust-analyzer
          ];

        craneLib = crane.overrideToolchain toolchain;
      in {
        devShells.default = craneLib.devShell {
          packages = with pkgs; [ toolchain hyperfine ];

          env = { LAZYVIM_RUST_DIAGNOSTICS = "bacon-ls"; };
        };
      });
}

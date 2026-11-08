{
  description = "Garuda Toolbox: dev shell and checks for the Garuda Linux maintenance and settings app (Rust, egui)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      treefmt-nix,
      git-hooks,
    }:
    let
      inherit (nixpkgs) lib;

      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: lib.genAttrs systems (system: f (perSystem system));

      perSystem =
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          rustToolchain = pkgs.callPackage ./nix/rust-toolchain.nix { };
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };

          treefmt = import ./nix/formatter.nix { inherit pkgs treefmt-nix rustToolchain; };
          packages = import ./nix/packages.nix { inherit pkgs rustPlatform rustToolchain; };
          devshell = import ./nix/devshell.nix {
            inherit pkgs rustToolchain;
            treefmtWrapper = treefmt.config.build.wrapper;
            preCommitHook = preCommit.shellHook;
          };
          preCommit = import ./nix/git-hooks.nix {
            inherit pkgs treefmt;
            gitHooksLib = git-hooks.lib;
            src = ./.;
          };
        in
        {
          inherit pkgs treefmt;
          inherit (packages) rustPkg lintEnv;
          inherit (devshell) devShellWith;
          checks = import ./nix/checks.nix {
            inherit pkgs self;
            inherit (packages) rustPkg;
            inherit rustToolchain treefmt;
            src = ./.;
          };
        };
    in
    {
      packages = forAllSystems (
        { rustPkg, lintEnv, ... }:
        {
          default = rustPkg.package;
          lint-env = lintEnv;
        }
      );

      formatter = forAllSystems ({ treefmt, ... }: treefmt.config.build.wrapper);

      devShells = forAllSystems (
        { pkgs, devShellWith, ... }:
        {
          default = devShellWith pkgs.stdenv.cc "";
          clang = devShellWith pkgs.clang ''
            export CC=clang CXX=clang++
            export CARGO_TARGET_${pkgs.stdenv.hostPlatform.rust.cargoEnvVarTarget}_LINKER=clang
          '';
        }
      );

      checks = forAllSystems ({ checks, ... }: checks);
    };
}

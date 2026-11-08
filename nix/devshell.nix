# clang comes through CC/CXX and the linker, because clangStdenv puts its build-side gcc first on PATH.
{
  pkgs,
  rustToolchain,
  treefmtWrapper,
  preCommitHook,
}:
let
  inherit (pkgs) lib;

  devLoop = pkgs.writeShellApplication {
    name = "start-dev";
    runtimeInputs = [ pkgs.git ];
    text = ''
      root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
      exec "$root/scripts/dev.sh" "$@"
    '';
  };

  devShellWith =
    compiler: compilerEnv:
    pkgs.mkShell {
      packages = [
        compiler
        pkgs.nixfmt
        pkgs.statix
        pkgs.deadnix
        pkgs.wayland
        pkgs.pkg-config
        # libalpm's major version must match the alpm crate's.
        pkgs.pacman
        rustToolchain
        pkgs.cargo-deny
        pkgs.cargo-nextest
        pkgs.cargo-llvm-cov
        pkgs.watchexec
        pkgs.git-lfs
        pkgs.weston
        pkgs.kdePackages.breeze-icons
        devLoop
        treefmtWrapper
      ];

      shellHook = ''
        # The IDE's fish integration fails on the exported name variable.
        unset name
        ${compilerEnv}
        ${preCommitHook}
        export LD_LIBRARY_PATH="${
          lib.makeLibraryPath (pkgs.callPackage ./runtime-libraries.nix { })
        }''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
        # The desktop's icon themes live in the system's data folders, which the shell drops.
        export XDG_DATA_DIRS="''${XDG_DATA_DIRS:+$XDG_DATA_DIRS:}/run/current-system/sw/share:/usr/local/share:/usr/share"
      '';
    };
in
{
  inherit devShellWith;
}

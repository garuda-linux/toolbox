{
  pkgs,
  rustPlatform,
  rustToolchain,
}:
{
  rustPkg = pkgs.callPackage ./package.nix {
    inherit rustPlatform;
    clippy = rustToolchain;
  };

  lintEnv = pkgs.buildEnv {
    name = "garuda-toolbox-lint-env";
    paths = [
      rustToolchain
      pkgs.nixfmt
      pkgs.statix
      pkgs.deadnix
    ];
  };
}

{
  pkgs,
  gitHooksLib,
  treefmt,
  src,
}:
gitHooksLib.${pkgs.stdenv.hostPlatform.system}.run {
  inherit src;
  package = pkgs.prek;
  hooks = {
    commitizen.enable = true;
    trufflehog.enable = true;

    treefmt = {
      enable = true;
      packageOverrides.treefmt = treefmt.config.build.wrapper;
    };

    clippy-workspace = {
      enable = true;
      entry = "cargo clippy --workspace --all-targets";
      files = "\\.rs$";
      pass_filenames = false;
    };

    toolbox-lint = {
      enable = true;
      entry = "scripts/lint.sh";
      files = "\\.rs$";
      pass_filenames = true;
    };

    file-size = {
      enable = true;
      entry = "scripts/check-file-size.sh .";
      always_run = true;
      pass_filenames = false;
    };
  };
}

{
  pkgs,
  self,
  src,
  rustPkg,
  rustToolchain,
  treefmt,
}:
let
  # The sandbox has no network, so Cargo uses the package's vendored crates.
  cargo-home = ''
    export HOME=$TMPDIR CARGO_HOME=$TMPDIR/cargo
    mkdir -p "$CARGO_HOME"
    cat > "$CARGO_HOME/config.toml" <<EOF
    [source.crates-io]
    replace-with = "vendored-sources"
    [source.vendored-sources]
    directory = "${rustPkg.package.cargoDeps}"
    EOF
  '';

  workspace = ''
    cp -r ${src}/Cargo.toml ${src}/Cargo.lock ${src}/rustfmt.toml ${src}/.cargo ${src}/app ${src}/crates .
    mkdir -p data
  '';

  xtaskCheck =
    name: setup: script:
    pkgs.runCommand name
      {
        nativeBuildInputs = [
          rustToolchain
          pkgs.pkg-config
        ];
        buildInputs = [ pkgs.pacman ];
      }
      ''
        ${cargo-home}
        ${setup}
        chmod -R u+w .
        ${script}
        touch $out
      '';
in
{
  formatting = treefmt.config.build.check self;

  clippy = rustPkg.check;

  tests = pkgs.runCommand "check-tests" { passthru.report = rustPkg.testReport; } ''
    status=$(cat ${rustPkg.testReport}/status)
    if [ "$status" != 0 ]; then
      echo "the tests failed (nextest exit $status): nix log ${rustPkg.testReport}" >&2
      exit 1
    fi
    touch $out
  '';

  file-size = pkgs.runCommand "check-file-size" { } ''
    cp -r ${src}/app app
    cp -r ${src}/crates crates
    cp ${src}/.file-size-allow .file-size-allow
    ${pkgs.bash}/bin/bash ${src}/scripts/check-file-size.sh .
    touch $out
  '';

  lint = pkgs.runCommand "check-lint" { } ''
    cp -r ${src}/app app
    cp -r ${src}/crates crates
    ${rustPkg.lint}/bin/toolbox-lint app crates
    touch $out
  '';

  cargo-deny =
    pkgs.runCommand "check-cargo-deny"
      {
        nativeBuildInputs = [
          rustToolchain
          pkgs.cargo-deny
        ];
      }
      ''
        ${cargo-home}
        cp ${src}/Cargo.toml ${src}/Cargo.lock ${src}/deny.toml .
        cp -r ${src}/app app
        cp -r ${src}/crates crates
        chmod -R u+w .
        cargo deny --offline check licenses bans sources
        touch $out
      '';

  # libalpm for the repository filter of `wiki-sync`.
  catalog =
    xtaskCheck "check-catalog"
      ''
        ${workspace}
        cp -r ${src}/data/catalog data/
        cp -r ${src}/i18n i18n
      ''
      ''
        cargo xtask catalog-validate
        cargo xtask catalog-ftl --check
        cargo xtask catalog-readable --check
        cargo xtask i18n-export --check
        cargo xtask i18n-import --check
      '';

  cli-docs = xtaskCheck "check-cli-docs" ''
    ${workspace}
    cp -r ${src}/data/cli data/
  '' "cargo xtask cli-docs --check";

  icons = xtaskCheck "check-icons" ''
    ${workspace}
    cp -r ${src}/data/catalog data/
  '' "cargo xtask icons-check ${pkgs.kdePackages.breeze-icons}/share/icons/breeze";

  # The whole checkout, because the tests read the catalogs.
  xtask-tests = xtaskCheck "check-xtask-tests" ''
    export CARGO_TARGET_DIR=$TMPDIR/target
    cp -r ${src}/. .
  '' "cargo test --offline -p toolbox-xtask";
}

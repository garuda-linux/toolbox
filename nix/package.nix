{
  lib,
  callPackage,
  rustPlatform,
  kdePackages,
  clippy,
  makeWrapper,
  pkg-config,
  pacman,
  wayland,
}:

let
  version = "0.1.0";

  # The tests take the same list, since a plain `cargo test` builds every member.
  appCrates = [
    "--workspace"
    "--exclude"
    "toolbox-lint"
    "--exclude"
    "toolbox-xtask"
  ];

  testCrates = appCrates ++ [
    "--features"
    "toolbox-gui/testing"
  ];

  runtimeLibraries = callPackage ./runtime-libraries.nix { };

  common =
    {
      withClippy,
      withTests ? false,
    }:
    rustPlatform.buildRustPackage {
      pname = "garuda-toolbox";
      inherit version;

      src = ../.;

      cargoLock = {
        lockFile = ../Cargo.lock;
      };

      cargoBuildFlags = appCrates;
      cargoTestFlags = testCrates;

      doCheck = withTests;
      useNextest = true;
      cargoCheckType = "debug";
      env.NEXTEST_PROFILE = "ci";

      nativeBuildInputs = [
        makeWrapper
        pkg-config
      ]
      ++ lib.optionals withClippy [ clippy ];

      buildInputs = [
        wayland
        pacman
      ];

      postInstall = ''
        staged=$(mktemp -d)
        mv $out/bin/garuda-toolbox $staged/
        mv $out/bin/garuda-toolbox-helper $staged/
        rm -f $out/bin/*
        bash scripts/install.sh "" $out $staged
      '';

      # Only the app is wrapped. The helper, run via pkexec, must stay a plain binary.
      postFixup = ''
        wrapProgram $out/bin/garuda-toolbox \
          --set GARUDA_TOOLBOX_CATALOG_DIR $out/share/garuda-toolbox/catalog \
          --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath runtimeLibraries} \
          --suffix XDG_DATA_DIRS : ${kdePackages.breeze-icons}/share
      '';

      meta = {
        description = "Maintain and configure Garuda Linux and other Arch based systems";
        mainProgram = "garuda-toolbox";
        homepage = "https://gitlab.com/garuda-linux/applications/toolbox";
        license = lib.licenses.gpl3Plus;
        platforms = lib.platforms.linux;
      };
    };
in
{
  package = common { withClippy = false; };

  lint = rustPlatform.buildRustPackage {
    pname = "toolbox-lint";
    inherit version;
    src = ../.;
    cargoLock.lockFile = ../Cargo.lock;
    cargoBuildFlags = [
      "-p"
      "toolbox-lint"
    ];
    cargoTestFlags = [
      "-p"
      "toolbox-lint"
    ];
    meta.mainProgram = "toolbox-lint";
  };

  check =
    let
      base = common { withClippy = true; };
    in
    base.overrideAttrs (_old: {
      name = "garuda-toolbox-check";

      buildPhase = ''
        runHook preBuild
        cargo clippy --offline --workspace --all-targets --features toolbox-gui/testing -- -D warnings
        runHook postBuild
      '';

      installPhase = ''
        mkdir -p $out
        touch $out/done
      '';

      postFixup = "";
    });

  # A failing run still builds, so CI can show its report. The `tests` check fails on the status.
  testReport =
    (common {
      withClippy = false;
      withTests = true;
    }).overrideAttrs
      (_old: {
        name = "garuda-toolbox-test-report";

        buildPhase = ''
          runHook preBuild
          runHook postBuild
        '';

        # Not the nextest hook, which stops the build when a test fails. nextest skips doc tests.
        checkPhase = ''
          runHook preCheck
          status=0
          cargo nextest run --offline --show-progress=none -j "$NIX_BUILD_CORES" \
            ${lib.escapeShellArgs testCrates} || status=$?
          cargo test --doc --offline -j "$NIX_BUILD_CORES" \
            ${lib.escapeShellArgs testCrates} || status=$?
          runHook postCheck
        '';

        installPhase = ''
          mkdir -p $out
          echo "''${status:-1}" > $out/status
          if [ -e target/nextest/ci/junit.xml ]; then
            cp target/nextest/ci/junit.xml $out/
          fi
        '';

        postFixup = "";
      });
}

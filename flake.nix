{
  description = "Garuda Toolbox";
  inputs = {
    devshell = {
      url = "github:numtide/devshell";
      flake = false;
    };
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    systems.url = "github:nix-systems/default";
  };

  outputs =
    inputs@{
      nixpkgs,
      self,
      systems,
      treefmt-nix,
      ...
    }:
    let
      eachSystem = f: nixpkgs.lib.genAttrs (import systems) (system: f nixpkgs.legacyPackages.${system});
      treefmtConfig = {
        programs = {
          deadnix.enable = true;
          nixfmt.enable = true;
          prettier = {
            settings = {
              arrowParens = "always";
              bracketSpacing = true;
              editorconfig = true;
              embeddedLanguageFormatting = "auto";
              endOfLine = "lf";
              plugins = [
                "prettier-plugin-tailwindcss"
                "prettier-plugin-organize-attributes"
              ];
              printWidth = 120;
              quoteProps = "consistent";
              semi = true;
              singleQuote = true;
              tabWidth = 2;
              trailingComma = "all";
              useTabs = false;
            };
            enable = true;
          };
          shellcheck.enable = true;
          shfmt.enable = true;
          statix = {
            disabled-lints = [ "repeated_keys" ];
            enable = true;
          };
          yamlfmt = {
            enable = true;
            settings.exclude = [
              "pnpm-lock.yaml"
            ];
          };
        };
      };
      treefmtEval = eachSystem (pkgs: treefmt-nix.lib.evalModule pkgs treefmtConfig);
    in
    {
      devShells = eachSystem (
        pkgs:
        let
          treefmtEval = treefmt-nix.lib.evalModule pkgs treefmtConfig;
          makeDevshell = import "${inputs.devshell}/modules" pkgs;
          mkShell =
            config:
            (makeDevshell {
              configuration = {
                inherit config;
                imports = [ ];
              };
            }).shell;
        in
        {
          default = mkShell {
            devshell = {
              name = "Toolbox shell";
              startup.preCommitHooks.text = self.checks.${pkgs.system}.pre-commit-check.shellHook + ''
                FLAKE_ROOT=$(${nixpkgs.lib.getExe pkgs.gitMinimal} rev-parse --show-toplevel)
                SYMLINK_SOURCE_PATH="${treefmtEval.config.build.configFile}"
                SYMLINK_TARGET_PATH="$FLAKE_ROOT/.treefmt.toml"

                if [[ -e "$SYMLINK_TARGET_PATH" && ! -L "$SYMLINK_TARGET_PATH" ]]; then
                  echo "treefmt-nix: Error: Target exists but is not a symlink."
                  exit 1
                fi

                if [[ -L "$SYMLINK_TARGET_PATH" ]]; then
                  if [[ "$(readlink "$SYMLINK_TARGET_PATH")" != "$SYMLINK_SOURCE_PATH" ]]; then
                    echo "treefmt-nix: Removing existing symlink"
                    unlink "$SYMLINK_TARGET_PATH"
                  else
                    exit 0
                  fi
                fi

                nix-store --add-root "$SYMLINK_TARGET_PATH" --indirect --realise "$SYMLINK_SOURCE_PATH"
                echo "treefmt-nix: Created symlink successfully"
              '';
            };
            env = [
              {
                name = "NIX_PATH";
                value = "${nixpkgs}";
              }
            ];
          };
          commands = [
            {
              package = pkgs.corepack;
            }
          ];
        }
      );

      formatter = eachSystem (pkgs: treefmtEval.${pkgs.system}.config.build.wrapper);

      checks = eachSystem (pkgs: {
        pre-commit-check = inputs.git-hooks.lib.${pkgs.system}.run {
          hooks = {
            check-json.enable = true;
            check-yaml = {
              enable = true;
              excludes = [
                "pnpm-lock.yaml"
              ];
            };
            commitizen.enable = true;
            detect-private-keys.enable = true;
            eslint = {
              enable = true;
              settings.extensions = "\\.(ts|js|mjs|html)$";
            };
            oxlint = {
              enable = true;
              name = "oxlint";
              entry = "${pkgs.oxlint}/bin/oxlint";
              files = "\\.(ts|js)$";
              language = "system";
              pass_filenames = false;
              stages = [ "pre-commit" ];
            };
            ripsecrets.enable = true;
            treefmt = {
              enable = true;
              args = [
                "--config-file"
                ".treefmt.toml"
              ];
            };
          };
          src = ./.;
        };
      });
    };
}

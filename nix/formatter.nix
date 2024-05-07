{ inputs, ... }: {
  imports = [
    inputs.treefmt-nix.flakeModule
  ];
  perSystem = { config, pkgs, ... }: {
    treefmt.config =
      {
        inherit (config.flake-root) projectRootFile;
        package = pkgs.treefmt;

        programs = {
          nixpkgs-fmt.enable = true;
          deadnix.enable = true;
          statix.enable = true;
          prettier.enable = true;
          rustfmt.enable = true;
          shellcheck.enable = true;
          shfmt.enable = true;
          ruff.format = true;
          ruff.check = true;
        };
        settings.formatter.prettier.includes = [ "*.yaml" "*.yml" ];
      };

    formatter = config.treefmt.build.wrapper;

    devshells.default = {
      commands = [
        {
          name = "fmt";
          help = "Format the repo";
          command = "nix fmt";
        }
      ];
    };
  };
}

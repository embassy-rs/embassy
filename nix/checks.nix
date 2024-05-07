{ lib, inputs, ... }: {
  imports = [
    inputs.devshell.flakeModule
  ];
  perSystem = { self', ... }:
    {
      checks =
        let
          # merge in any devshells or packages defined by the flake so they get built as part of `nix flake check`
          devShells = with lib; mapAttrs' (n: nameValuePair "devShell-${n}") self'.devShells;
          packages = with lib; mapAttrs' (n: nameValuePair "package-${n}") self'.packages;
        in
        devShells // packages;

      devshells.default = {
        commands = [
          {
            name = "check";
            help = "Run flake checks";
            command = "nix flake check";
          }
        ];
      };
    };
}

{ inputs, ... }: {
  imports = [
    inputs.flake-root.flakeModule
    ./devshell.nix
    ./checks.nix
    ./formatter.nix
    ./examples.nix
  ];
}

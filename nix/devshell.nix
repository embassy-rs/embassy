{ inputs, ... }: {
  imports = [
    inputs.devshell.flakeModule
  ];

  perSystem = { system, ... }:
    let
      overlays = [ (import inputs.rust-overlay) ];
      pkgs = import inputs.nixpkgs {
        inherit system overlays;
      };
      rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ../rust-toolchain-nightly.toml;
    in
    {
      devshells.default = {
        commands = [
          {
            package = pkgs.cachix;
          }
          {
            package = pkgs.callPackage ./probe-rs.nix { };
          }
          {
            package = rustToolchain;
          }
        ];
      };
    };
}

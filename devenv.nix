{
  pkgs,
  config,
  ...
}:
{
  packages = [
    pkgs.flip-link
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
    version = "2025-12-11";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-src"
      "llvm-tools"
      "rust-analyzer"
    ];
    lsp.enable = false; # disable nixpkgs rust-analyzer
    targets = [
      "x86_64-unknown-linux-gnu"
      "thumbv6m-none-eabi"
      "thumbv7m-none-eabi"
      "thumbv7em-none-eabi"
      "thumbv7em-none-eabihf"
      "thumbv8m.main-none-eabihf"
      "riscv32imac-unknown-none-elf"
      "wasm32-unknown-unknown"
      "armv7a-none-eabi"
      "armv7r-none-eabi"
      "armv7r-none-eabihf"
    ];
  };

  treefmt = {
    enable = true;
    config.programs = {
      nixfmt.enable = true;
      rustfmt = {
        enable = true;
        package = config.languages.rust.toolchain.rustfmt;
        edition = "2024";
      };
    };
  };
}

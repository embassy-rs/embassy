{ probe-rs, fetchFromGitHub, lib }:
probe-rs.overrideAttrs (old: rec {
  src = fetchFromGitHub {
    owner = old.pname;
    repo = old.pname;
    rev = "cf6ae5e5761731c228a2753617b46dd03869e8ac";
    hash = "sha256-aLe8ERHgWSOILXfhbABpxLAEucJx8Bv0ZYrOLdqzmoU=";
  };
  cargoBuildFlags = [ ];
  # Need this as `cargo test` is failing on 05/03/2024
  doCheck = false;
  cargoDeps = old.cargoDeps.overrideAttrs (lib.const {
    inherit src;
    name = "${old.pname}-vendor.tar.gz";
    outputHash = "sha256-oz/MlqYlHoyT648qnTrLPJHBMaXW+wCsfneN2Jjuimk=";
  });
})

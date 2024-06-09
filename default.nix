{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage {
  pname = "nyaa";
  version =
    (builtins.fromTOML (
      builtins.readFile ./Cargo.toml
    ))
    .package
    .version;

  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
}

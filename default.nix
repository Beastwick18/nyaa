{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "nyaa";
  version = "0.1";
  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "transmission-rpc-0.4.2" = "sha256-3na/LUlcCk1MTbFXMtugK8YiLMdwF3m8QnpSKl3B4I8=";
    };
  };
  src = pkgs.lib.cleanSource ./.;
}

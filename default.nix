{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "nyaa";
  version = "0.9.0";

  # src = fetchFromGitHub {
  #   owner = "Beastwick18";
  #   repo = "nyaa";
  #   rev = "v${version}";
  #   hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  # };

  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
}

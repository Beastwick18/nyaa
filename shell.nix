{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  inputsFrom = [(pkgs.callPackage ./default.nix {})];
  buildInputs =  [
    pkgs.rustPackages.clippy
  ];
}

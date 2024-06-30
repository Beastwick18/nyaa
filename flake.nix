{
  description = "A nyaa.si tui tool for browsing and downloading torrents";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
		naersk.url = "github:nix-community/naersk";
  };
  outputs = {
    self,
    nixpkgs,
		naersk,
  }@inputs: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
		system = "x86_64-linux";
		pkgs = import nixpkgs {
			inherit system;
		};
		naersk' = pkgs.callPackage naersk {};
  in {
    packages = forAllSystems (system: {
			default = naersk'.buildPackage {
				name = "nyaa";
				version =
					(builtins.fromTOML (
						builtins.readFile ./Cargo.toml
					))
					.package
					.version;
				src = pkgs.lib.cleanSource ./.;
			};
    });

    devShells = forAllSystems (system: {
			default = pkgs.mkShell {
				nativeBuildInputs = with pkgs; [ rust.packages.stable.rustPlatform.rustcSrc rustc cargo rustPackages.clippy docker ];
			};
    });

    homeManagerModule = import ./modules/home-manager.nix inputs;
  };
}

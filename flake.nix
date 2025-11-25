{
  description = "A Wayland client application with rounded screen corners";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShell = pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.rustup
            pkgs.wayland
          ];

          shellHook = ''
            rustup default stable
            rustup target add x86_64-unknown-linux-gnu
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rounded-screen";
          version = "0.1.0";

          src = ./.;

          cargoHash = "sha256-MDX+jyodV1lhug1DJVgPuRQNNbrkNKzzOkpVfHfG3oQ=";

          nativeBuildInputs = [
            pkgs.wayland
          ];
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/rounded-screen";
        };
      }
    );
}

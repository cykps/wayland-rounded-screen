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
            pkgs.pkg-config
            pkgs.wayland

            # Required for building smithay-client-toolkit
            pkgs.libxkbcommon
            pkgs.pkg-config
          ];

          shellHook = ''
            rustup default stable
            rustup target add x86_64-unknown-linux-gnu
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rounded-screan";
          version = "0.1.0";

          src = ./.;

          cargoSha256 = "sha256-0000000000000000000000000000000000000000000000000000"; # Replace with the actual hash

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.wayland

            # Required for building smithay-client-toolkit
            pkgs.libxkbcommon
            pkgs.pkg-config
          ];
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/rounded-screan";
        };
      }
    );
}

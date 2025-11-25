{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo

    # wayland
    # wayland-protocols

    # Required for building smithay-client-toolkit
    libxkbcommon
    pkg-config
  ];
}

{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust
    rustc
    cargo
    pkg-config

    # wayland
    wayland
    wayland-protocols
    libxkbcommon
  ];
}

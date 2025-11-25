# wayland-rounded-screen

A Wayland client application that renders rounded corner overlays on the screen.


## Features

- Draws rounded corner overlays on Wayland screens.
- Configurable radius for rounded corners.


## System Dependencies

- **Rust** (with Cargo)
- **Wayland**


## Installation

### Using Nix Flake

#### Prerequisites

- Install [Nix](https://nixos.org/download.html) on your system.

#### Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/cykps/wayland-rounded-screen.git
   cd wayland-rounded-screen
   ```

2. Enter the development shell:
   ```bash
   nix develop
   ```

3. Build the project:
   ```bash
   cargo build --release
   ```

4. Run the application:
   ```bash
   ./target/release/rounded-screan
   ```


## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.


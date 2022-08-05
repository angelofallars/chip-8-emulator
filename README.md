# CHIP-8 emulator

A CHIP-8 emulator written in Rust. Currently has a basic GUI for displaying the screen.

## Build

Requires: Rust

```bash
git clone https://github.com/angelofallars/brainfrick-rs
cd chip-8-emulator
cargo install --path .
```

## Runtime requirements

```bash
# Arch-based (Arch / Manjaro / Endeavour) requirements
sudo pacman -S pkg-config libx11 libxi mesa-libgl alsa-lib

# Ubuntu-based (Ubuntu / Mint / Pop!_OS) requirements
sudo apt install pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev

# Fedora requirements
sudo dnf install libX11-devel libXi-devel mesa-libGL-devel alsa-lib-devel
```

## License

MIT

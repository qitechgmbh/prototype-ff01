{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Core Rust toolchain
    rustc
    cargo
    rustfmt
    clippy

    # System libraries
    glibc
    udev

    # Common native deps for Rust crates
    pkg-config
    openssl
    cmake
    gcc

    # Optional but commonly needed
    zlib
  ];

  # Fix for many crates needing pkg-config
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
}

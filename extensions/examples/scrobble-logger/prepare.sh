#!/bin/bash

# Function to check if a command exists
command_exists () {
  command -v "$1" >/dev/null 2>&1
}

missing_deps=0

# Check for Cargo
if ! (command_exists cargo); then
  missing_deps=1
  echo "❌ Cargo/rust is not installed."
  echo ""
  echo "To install Rust, visit the official download page:"
  echo "👉 https://www.rust-lang.org/tools/install"
  echo ""
  echo "Or install it using a package manager:"
  echo ""
  echo "🔹 macOS (Homebrew):"
  echo "    brew install cargo"
  echo ""
  echo "🔹 Ubuntu/Debian:"
  echo "    sudo apt-get install -y cargo"
  echo ""
  echo "🔹 Arch Linux:"
  echo "    sudo pacman -S rust"
  echo ""
fi

if ! (command_exists rustup); then
  missing_deps=1
  echo "❌ rustup is missing. Check your rust installation."
  echo ""
fi

# Exit with a bad exit code if any dependencies are missing
if [ "$missing_deps" -ne 0 ]; then
  echo "Install the missing dependencies and ensure they are on your path. Then run this command again."
  # TODO: remove sleep when cli bug is fixed
  sleep 2
  exit 1
fi

if ! (rustup target list --installed | grep -q '^wasm32-wasip1$'); then
  if ! (rustup target add wasm32-wasip1); then
    echo "❌ error encountered while adding target \"wasm32-wasip1\""
    echo ""
    echo "Update rustup with:"
    echo "👉 rustup update"
    echo ""
    exit 1
  fi
fi

if ! (rustup target list --installed | grep -q '^wasm32-unknown-unknown$'); then
  rustup target add wasm32-unknown-unknown
fi

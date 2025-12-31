#!/bin/bash
set -e

echo "Building Doublets WebAssembly Demo..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WebAssembly module
echo "Building WebAssembly module..."
wasm-pack build --target web --out-dir pkg

echo "Build complete! The demo is ready to serve."
echo "To run the demo locally:"
echo "  cd web-demo"
echo "  python3 -m http.server 8000"
echo "  # Then open http://localhost:8000 in your browser"
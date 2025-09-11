# Doublets Web Demo

An interactive WebAssembly playground for exploring doublets operations in your browser.

## Features

- **Create Links**: Build custom doublets with specified source and target
- **Create Points**: Generate self-referencing links (points)
- **Delete Links**: Remove links by ID
- **Search & Filter**: Find links by source and/or target
- **Real-time Display**: Live view of all links in the store
- **Operations Log**: Track all operations performed

## Quick Start

### Prerequisites

- Rust toolchain with WebAssembly target
- `wasm-pack` tool for building WebAssembly modules

### Building

1. Install the WebAssembly target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. Install wasm-pack (if not already installed):
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

3. Build the demo:
   ```bash
   cd web-demo
   ./build.sh
   ```

### Running

Start a local web server:
```bash
python3 -m http.server 8000
```

Then open [http://localhost:8000](http://localhost:8000) in your browser.

## Usage

### Creating Links

1. **Points**: Click "Create Point" to create a self-referencing link (1 → 1)
2. **Custom Links**: Enter source and target values, then click "Create Link"

### Managing Links

- View all links in the table on the right
- Delete links by entering their ID and clicking "Delete Link"
- Search for specific links using source/target filters

### Understanding Doublets

Doublets are a fundamental data structure where:
- Each **link** connects a **source** to a **target**
- Links themselves have unique **IDs**
- **Points** are special links where source equals target
- The system maintains referential integrity

## Architecture

The demo consists of:

- **Rust/WebAssembly Core**: Uses the `doublets` crate for all operations
- **JavaScript Interface**: Provides UI interactions and state management
- **HTML/CSS Frontend**: Responsive web interface

## Example Operations

```javascript
// Create a new doublets store
const demo = new DoubletsDemo();

// Create a point (self-link)
const point = demo.create_point(); // Returns: 1

// Create a custom link
const link = demo.create_link(2, 3); // Returns: 2

// Get all links
const allLinks = demo.get_all_links();
// Returns: [
//   { id: 1, source: 1, target: 1 },
//   { id: 2, source: 2, target: 3 }
// ]

// Search for links
const results = demo.search_links(2, null); // Find links with source=2
```

## Contributing

This demo is part of the [doublets-rs](https://github.com/linksplatform/doublets-rs) project. Contributions are welcome!
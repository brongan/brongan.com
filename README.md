# Brongan.com

Welcome to the source code for [Brongan.com](https://brongan.com), a personal playground for exploring Rust, WebAssembly, and systems programming in the browser.

This project is built using the [Leptos](https://github.com/leptos-rs/leptos) full-stack framework and [Axum](https://github.com/tokio-rs/axum).

## 🚀 Features

### 🖥️ Chip-8 Emulator
A fully functional Chip-8 interpreter written in Rust and running in WebAssembly.
- **Cycle-accurate execution**: Runs standard ROMs like Pong, Brix, and Tetris.
- **Debugger**: detailed view of registers, memory, and stack.
- **Disassembler**: Real-time instruction decoding.
- **Keypad**: Interactive on-screen keypad with keyboard support.

### 🧬 Conway's Game of Life
A high-performance implementation of the Game of Life using WebGL.
- **WebGL Rendering**: Renders the universe grid directly on the GPU for performance.
- **Interactivity**: Click to toggle cells, Shift+Click for Pulsars, Ctrl+Click for Gliders.
- **Simulation Control**: Start, stop, step, and reset the simulation.

### 👁️ Ishihara Test Generator
A tool to generate Color Blindness tests on the fly.
- **Algorithmic Generation**: Uses circle packing algorithms to create plates.
- **Customizable**: Inputs for text and blindness types (Red-Green, Blue-Yellow).

### 🎨 Other Experiments
- **Mandelbrot Explorer**: Fractal visualization.
- **Catscii**: Image-to-ASCII art converter.
- **Analytics**: A privacy-first, custom analytics solution tracking country-level traffic.

## 🛠️ Technology Stack

- **Frontend**: [Leptos](https://leptos.dev) (Rust -> WASM)
- **Backend**: [Axum](https://github.com/tokio-rs/axum)
- **Styling**: SCSS (compiled via `cargo-leptos`)
- **Database**: SQLite / GeoLite2 (for analytics)
- **Build Tool**: `cargo-leptos` / `just`

## 🏃 Local Development

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (Nightly toolchain required)
- `cargo-leptos`: `cargo install cargo-leptos`
- `sass`: `npm install -g sass` (or your preferred method)

### Running the App

The easiest way to run the app is using the `just` command runner, but you can also use `cargo-leptos` directly.

**Using Just:**
```bash
# Start the development server (auto-reloads)
just develop

# Build for production
just build
```

**Using Cargo Leptos:**
```bash
cargo leptos watch
```

The app will be available at `http://localhost:3000`.

## 🐳 Deployment

The application is containerized using Docker and deployed to [Fly.io](https://fly.io).

```bash
# Build container image
just container

# Deploy to Fly.io
just deploy
```

## 📄 License

This project is open-source. Feel free to explore the code!

# swcat - Cinematic Text Crawler

`swcat` is a Rust-based command-line tool that renders text files as a cinematic Star Wars opening crawl. It features a full intro sequence, including a prologue, a hollow receding logo, and a 3D perspective text crawl with a randomized starfield.

## Project Overview

- **Purpose**: A visually impressive alternative to `cat` for viewing text or code files.
- **Technologies**: 
  - **Rust**: Language.
  - **Macroquad**: Game engine used for 2D/3D rendering, windowing, and shaders.
  - **Clap**: CLI argument parsing.
  - **GLSL**: Custom shaders for specialized visual effects (e.g., the "hollow" logo).
- **Architecture**:
  - **State Machine**: The app transitions through `Prologue`, `Logo`, and `Crawl` states.
  - **Multi-Chunk Rendering**: To support long files (like source code) without hitting GPU texture limits, text is split into vertical chunks, each rendered to its own texture and placed as a separate segment in 3D space.
  - **Dynamic Scaling**: The 3D plane and texture width scale dynamically based on the longest line in the input file.

## Building and Running

### Key Commands

- **Build**: `cargo build`
- **Run**: `cargo run -- <file_path> [options]`
- **Install**: `cargo install --path .`

### CLI Options

- `<file_path>`: The path to the text file to crawl.
- `--speed <f32>`: Sets the crawl speed (default: 50.0).
- `--skip-intro`: Skips the "A long time ago..." and logo sequences.
- `--left`: Aligns the crawl text to the left (ideal for code).
- `--width <usize>` / `-w`: Limits the line width (truncates by default).
- `--wrap` / `-W`: Enables word wrapping when used with `--width`.

## Development Conventions

- **Shaders**: Custom GLSL shaders are embedded as constants (`VERTEX_SHADER`, `FRAGMENT_SHADER`).
- **Rendering Logic**:
  - **Prologue**: Blue text, left-aligned.
  - **Logo**: Hollow yellow "SWCAT" logo using a thickness-detecting fragment shader over a transparent render target.
  - **Crawl**: Yellow text rendered onto `RenderTarget` chunks and mapped to 3D quads.
- **Coordinate Systems**:
  - Uses `Camera2D` for UI and off-screen rendering.
  - Uses `Camera3D` for the perspective crawl.
  - Note: `RenderTarget` mapping requires careful Y-axis flipping (`zoom: y = -2.0 / height`, `flip_y: true`).
- **Input**:
  - `Esc` or `q`: Exit.
  - Mouse Wheel: Manual scroll control.

## Asset Reference
- `sw_logo.jpg`: Reference for the "hollow" bubble style.
- `sw_screen_crawl.jpeg`: Reference for the 3D perspective and starfield.
- `crawl.txt`: Sample text for testing.

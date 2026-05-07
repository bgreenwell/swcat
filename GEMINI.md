# swcat - Cinematic Text Crawler

`swcat` is a Rust-based command-line tool that renders text files as a cinematic Star Wars opening crawl. It supports two modes: a terminal-based TUI (default) and a high-fidelity graphical GUI (`--gui`).

## Project Overview

- **Purpose**: A visually impressive alternative to `cat` for viewing text or code files.
- **Technologies**: 
  - **Rust**: Language.
  - **Ratatui / Crossterm**: Used for the default TUI mode.
  - **Macroquad**: Used for the GUI mode (2D/3D rendering, windowing, and shaders).
  - **Clap**: CLI argument parsing.
- **Architecture**:
  - **Modular Router**: `main.rs` parses arguments and routes execution to either the `tui` or `gui` modules.
  - **TUI Module (`src/tui.rs`)**: Uses a custom rendering loop to simulate 3D perspective with ASCII art (brackets) and dynamic centering.
  - **GUI Module (`src/gui.rs`)**: Uses a full 3D engine with GLSL shaders for the intro and crawl sequence.
  - **Shared Text Logic (`src/text.rs`)**: Handles tab expansion, width limiting, and word wrapping.

## Building and Running

### Key Commands

- **Build**: `cargo build`
- **Run (TUI)**: `cargo run -- <file_path> [options]`
- **Run (GUI)**: `cargo run -- --gui <file_path> [options]`
- **Install**: `cargo install --path .`

### CLI Options

- `<file_path>`: The path to the text file to crawl.
- `--gui`: Switches to graphical mode.
- `--speed <f32>`: Sets the crawl speed (TUI default: 3.0, GUI default: 50.0).
- `--skip-intro`: Skips the prologue and logo sequences.
- `--left`: Aligns the crawl text to the left (ideal for code). In TUI, the header remains centered.
- `--border` (TUI only): Draws bracket borders around the crawl.
- `--width <usize>` / `-w`: Limits the line width.
- `--wrap` / `-W`: Enables word wrapping.

## Development Conventions

- **TUI Rendering**:
  - Simulates perspective by "shrinking" the horizontal space available for text on higher screen rows.
  - **Dynamic Centering**: Headers and body text (unless `--left` is used) are centered frame-by-frame based on the narrowed viewport.
- **GUI Rendering**:
  - **Shaders**: Custom GLSL shaders are embedded as constants in `gui.rs`.
  - **Logo**: Hollow yellow logo using a thickness-detecting fragment shader.
  - **Multi-Chunk Rendering**: Split into vertical textures to avoid GPU limits.
- **State Machine**: Both modes follow a similar `Prologue` -> `Logo` -> `Crawl` lifecycle.

# swcat

A visually impressive alternative to `cat`.

![swcat demo](assets/swcat-demo.gif) Renders any text file as a cinematic Star Wars–style opening crawl, complete with a starfield, hollow receding logo, and 3D perspective scroll.

## Installation

```bash
cargo install --path .
```

Or run directly without installing:

```bash
cargo run -- <file> [options]
```

## Usage

```
swcat <file> [OPTIONS]
```

### Options

| Flag | Short | Description |
|---|---|---|
| `--speed <f32>` | `-s` | Crawl speed (default: `50.0`) |
| `--skip-intro` | | Skip the "A long time ago…" and logo sequences |
| `--left` | | Left-align text as a block (recommended for code) |
| `--width <n>` | `-w` | Truncate lines longer than `n` characters |
| `--wrap` | `-W` | Word-wrap instead of truncating (requires `--width`) |
| `--no-header` | | Omit the filename title at the top of the crawl |
| `--scale <f32>` | | Window size multiplier (default `1.0` = 1000×618) |

### Examples

```bash
# Crawl a prose file
swcat README.md

# Crawl source code, left-aligned, wrapped at 80 chars
swcat src/main.rs --left --width 80 --wrap

# Jump straight to the crawl
swcat notes.txt --skip-intro --speed 80
```

## Controls

| Key | Action |
|---|---|
| `Space` | Pause / resume |
| `↑` / `=` | Increase speed |
| `↓` / `-` | Decrease speed |
| Mouse wheel | Manual scroll |
| `q` / `Esc` | Quit |

## Building from source

Requires Rust (stable). No system dependencies beyond a GPU with OpenGL support.

```bash
cargo build --release
```

The binary lands at `target/release/swcat`.

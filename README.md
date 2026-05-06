# swcat

A visually impressive alternative to `cat`. Renders any text file as a cinematic Star Wars–style opening crawl, complete with a starfield, hollow receding logo, and 3D perspective scroll.

```bash
cargo run -- README.md --left --width 25 -W
```

![swcat demo](assets/swcat-demo.gif)

## Inspiration

Inspired by a [tweet from @github](https://x.com/github/status/2051436651881124044) where they asked Copilot CLI to build a Star Wars crawl from the GitHub Changelog — and it delivered. `swcat` takes that idea and runs with it as a general-purpose `cat` replacement.

## Requirements

- Rust (stable) — install via [rustup.rs](https://rustup.rs)
- A GPU with OpenGL support (any modern machine)

## Installation

Install directly from GitHub (no clone needed):

```bash
cargo install --git https://github.com/bgreenwell/swcat
```

Or clone and install locally:

```bash
git clone https://github.com/bgreenwell/swcat
cd swcat
cargo install --path .
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

```bash
git clone https://github.com/bgreenwell/swcat
cd swcat
cargo build --release
./target/release/swcat <file>
```

# swcat

Why read a file like a normal person? `swcat` renders any text file as a cinematic Star Wars–style opening crawl. By default, it runs in your terminal (TUI), but it also features a high-fidelity graphical mode (GUI) with starfields and hollow receding logos.

```bash
# Terminal (TUI) mode
swcat README.md

# Graphical (GUI) mode
swcat README.md --gui
```

![swcat demo](assets/swcat-demo.gif)

## Inspiration

Inspired by a [tweet from @github](https://x.com/github/status/2051436651881124044) where they asked Copilot CLI to build a Star Wars crawl from the GitHub Changelog — and it delivered. `swcat` takes that idea and runs with it — because sometimes `cat` just isn't dramatic enough.

## Requirements

- Rust (stable) — install via [rustup.rs](https://rustup.rs)
- **TUI Mode (Default):** Any standard terminal.
- **GUI Mode (`--gui`):** A GPU with OpenGL support.

## Installation

Install directly from GitHub:

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

### General Options

| Flag | Short | Description |
|---|---|---|
| `--gui` | | Run in graphical mode instead of the terminal |
| `--speed <f32>` | `-s` | Crawl speed (TUI default: `3.0`, GUI default: `50.0`) |
| `--skip-intro` | | Skip the "A long time ago…" and logo sequences |
| `--width <n>` | `-w` | Truncate lines longer than `n` characters |
| `--wrap` | `-W` | Word-wrap instead of truncating (requires `--width`) |
| `--no-header` | | Omit the filename title at the top of the crawl |

### Mode-Specific Options

| Flag | Mode | Description |
|---|---|---|
| `--border` | TUI | Show a bracket border around the text crawl |
| `--left` | GUI/TUI | Left-align body text (TUI header stays centered) |
| `--scale <f32>` | GUI | Window size multiplier (default `1.0` = 1000×618) |

### Examples

```bash
# Standard terminal crawl
swcat README.md

# Terminal crawl with borders and wrapping
swcat src/main.rs --border --width 80 --wrap

# High-fidelity GUI crawl
swcat README.md --gui

# GUI crawl, left-aligned code, fast speed
swcat src/lib.rs --gui --left --speed 100
```

## Controls

| Key | Action |
|---|---|
| `Space` | Pause / resume |
| `↑` / `=` | Increase speed |
| `↓` / `-` | Decrease speed |
| Mouse wheel | Manual scroll (GUI only) |
| `q` / `Esc` | Quit |

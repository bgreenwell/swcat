# Merge TUI and GUI Plan

## Objective
Combine the TUI and GUI versions of `swcat` into a single executable. Make the TUI the default mode and introduce a `--gui` flag to trigger the graphical mode. Ensure that the graphical window is only initialized when requested.

## Key Files & Context
- `src/main.rs`: The unified entry point and CLI argument parser.
- `src/gui.rs`: The new module containing the macroquad graphical implementation.
- `src/tui.rs`: The new module containing the crossterm text-based implementation.
- `src/lib.rs`: Needs to expose the new modules.
- `Cargo.toml`: Needs cleanup to remove the `swcat-tui` binary target.

## Implementation Steps

1. **Extract GUI Logic:**
   - Create `src/gui.rs`.
   - Move the graphical code (shaders, macroquad rendering loop, `window_conf`) from `src/main.rs` into `src/gui.rs`.
   - Remove the `#[macroquad::main]` macro.
   - Refactor the main async function in `src/gui.rs` to take the parsed arguments and execute the logic. `macroquad::Window::from_config` will be used to initialize the window.

2. **Extract TUI Logic:**
   - Rename `src/bin/swcat_tui.rs` to `src/tui.rs` (or create and copy).
   - Refactor it to export a function that takes the parsed arguments and executes the crossterm loop.

3. **Unify CLI Arguments (`src/main.rs`):**
   - Create a single `Args` struct in `src/main.rs`.
   - Add a `gui` boolean flag.
   - Change `speed` to `Option<f32>` to allow mode-specific defaults (e.g., 3.0 for TUI, 50.0 for GUI).
   - Add a standard synchronous `main` function.
   - Parse arguments, determine the target speed, and route execution to either `tui::run(...)` or `macroquad::Window::from_config(..., gui::run(...))`.

4. **Update `Cargo.toml` and `src/lib.rs`:**
   - Remove the `[[bin]]` section for `swcat-tui` from `Cargo.toml`.
   - Update `src/lib.rs` to declare the `tui` and `gui` modules.

## Verification & Testing
1. Run `cargo build` to ensure the refactored code compiles.
2. Run `cargo run -- <file>` to verify the TUI mode launches correctly by default and renders properly.
3. Run `cargo run -- --gui <file>` to verify the graphical window opens and renders the crawl correctly without issues.
4. Verify that running the TUI default does not flash an empty graphical window.
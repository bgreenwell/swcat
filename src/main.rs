use swcat::{gui, tui};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to read
    file: PathBuf,

    /// Speed of the crawl (default 3.0 for TUI, 50.0 for GUI)
    #[arg(short, long)]
    speed: Option<f32>,

    /// Skip the opening intro sequence
    #[arg(long)]
    skip_intro: bool,

    /// Maximum number of characters per line (truncates unless --wrap is set)
    #[arg(short, long)]
    width: Option<usize>,

    /// Wrap lines that exceed --width instead of truncating them
    #[arg(short = 'W', long)]
    wrap: bool,

    /// Omit the filename header at the top of the crawl
    #[arg(long)]
    no_header: bool,

    /// Run in graphical (GUI) mode instead of the default terminal (TUI) mode
    #[arg(long)]
    gui: bool,

    // -- GUI specific ---------------------------------------------------------

    /// [GUI] Left-align the text instead of centering it
    #[arg(long)]
    left: bool,

    /// [GUI] Window size multiplier (default 1.0 = 1000x618)
    #[arg(long, default_value_t = 1.0)]
    scale: f32,

    // -- TUI specific ---------------------------------------------------------

    /// [TUI] Show a bracket border around the text crawl
    #[arg(long)]
    border: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(s) = args.speed {
        if s < 0.0 {
            eprintln!("Error: --speed must be a positive number");
            std::process::exit(1);
        }
    }

    if args.gui {
        if args.scale <= 0.0 {
            eprintln!("Error: --scale must be a positive number");
            std::process::exit(1);
        }

        let speed = args.speed.unwrap_or(50.0);
        let gui_args = gui::GuiArgs {
            file: args.file,
            speed,
            skip_intro: args.skip_intro,
            left: args.left,
            width: args.width,
            wrap: args.wrap,
            no_header: args.no_header,
        };

        let conf = macroquad::window::Conf {
            window_title: "swcat".to_owned(),
            window_width: (gui::BASE_WINDOW_W as f32 * args.scale).round() as i32,
            window_height: (gui::BASE_WINDOW_H as f32 * args.scale).round() as i32,
            window_resizable: true,
            ..Default::default()
        };

        macroquad::Window::from_config(conf, gui::run(gui_args));
    } else {
        let speed = args.speed.unwrap_or(3.0);
        let tui_args = tui::TuiArgs {
            file: args.file,
            speed,
            skip_intro: args.skip_intro,
            width: args.width,
            wrap: args.wrap,
            no_header: args.no_header,
            border: args.border,
            left: args.left,
        };

        if let Err(e) = tui::run(tui_args) {
            eprintln!("TUI Error: {e}");
            std::process::exit(1);
        }
    }
}

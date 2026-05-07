use std::{
    io::stdout,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Position, Rect},
    prelude::Widget,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
    Frame, Terminal,
};

use swcat::text;

const PROLOGUE_MS: f32 = 4000.0;
const LOGO_MS: f32 = 6000.0;
const DEFAULT_SPEED: f32 = 3.0;  // lines per second
const SPEED_STEP: f32 = 0.5;
const MAX_SPEED: f32 = 20.0;
const STAR_COUNT: usize = 180;
const FRAME_MS: u64 = 33; // ~30 fps

const LOGO_ART: &str = "\
 ____  _    _  ____    _  _____ \n\
/ ___|| |  | |/ ___|  / \\|_   _|\n\
\\___ \\| |/\\| | |    / _ \\ | |  \n\
 ___) |  /\\  | |___ / ___ \\| |  \n\
|____/|_/  \\_|\\____/_/   \\_\\_|";

const PROLOGUE_TEXT: &str = "A long time ago in a terminal far,\n\nfar, far away....";

#[derive(Parser)]
#[command(name = "swcat-tui", about = "Star Wars crawl in your terminal")]
struct Args {
    file: PathBuf,
    #[arg(short = 's', long, default_value_t = DEFAULT_SPEED)]
    speed: f32,
    #[arg(long)]
    skip_intro: bool,
    #[arg(long)]
    left: bool,
    #[arg(short = 'w', long)]
    width: Option<usize>,
    #[arg(short = 'W', long)]
    wrap: bool,
    #[arg(long)]
    no_header: bool,
}

struct Star {
    row: u16,
    col: u16,
    ch: char,
}

enum AppState {
    Prologue { entered: Instant },
    Logo { entered: Instant, stars: Vec<Star> },
    Crawl { stars: Vec<Star>, scroll: f32, speed: f32, paused: bool },
    Done,
}

// Smooth fade envelope: in over first 30%, hold 40%, out over last 30%.
fn fade_alpha(elapsed_ms: f32, total_ms: f32) -> f32 {
    let t = (elapsed_ms / total_ms).clamp(0.0, 1.0);
    let fade_frac = 0.30;
    if t < fade_frac {
        t / fade_frac
    } else if t < 1.0 - fade_frac {
        1.0
    } else {
        (1.0 - t) / fade_frac
    }
}

fn lerp_color(a: f32, r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(
        (r as f32 * a) as u8,
        (g as f32 * a) as u8,
        (b as f32 * a) as u8,
    )
}

fn make_stars(rows: u16, cols: u16) -> Vec<Star> {
    let mut seed: u64 = 0xdeadbeef_cafe1234;
    let mut next = move || -> u64 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        seed >> 33
    };
    let chars = ['*', '.', '·', '+'];
    (0..STAR_COUNT)
        .map(|_| Star {
            row: (next() % rows.max(1) as u64) as u16,
            col: (next() % cols.max(1) as u64) as u16,
            ch: chars[(next() % chars.len() as u64) as usize],
        })
        .collect()
}

fn draw_stars(buf: &mut ratatui::buffer::Buffer, stars: &[Star], area: Rect) {
    for s in stars {
        if s.row < area.height && s.col < area.width {
            if let Some(cell) = buf.cell_mut(Position::new(s.col, s.row)) {
                cell.set_char(s.ch).set_fg(Color::DarkGray).set_bg(Color::Black);
            }
        }
    }
}

fn centered_sub(area: Rect, width_pct: u16, height: u16) -> Rect {
    let w = (area.width * width_pct / 100).max(1).min(area.width);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, w, height.min(area.height))
}

fn render_prologue(frame: &mut Frame, elapsed_ms: f32) {
    let area = frame.area();
    let a = fade_alpha(elapsed_ms, PROLOGUE_MS);
    let color = lerp_color(a, 0, 204, 255); // cyan
    let para = Paragraph::new(Text::from(PROLOGUE_TEXT).alignment(Alignment::Center))
        .style(Style::default().fg(color).bg(Color::Black))
        .alignment(Alignment::Center);
    let text_area = centered_sub(area, 60, 5);
    frame.render_widget(Paragraph::new("").style(Style::default().bg(Color::Black)), area);
    frame.render_widget(para, text_area);
}

fn render_logo(frame: &mut Frame, stars: &[Star], elapsed_ms: f32) {
    let area = frame.area();
    let buf = frame.buffer_mut();

    // Clear to black
    for y in 0..area.height {
        for x in 0..area.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_char(' ').set_fg(Color::Black).set_bg(Color::Black);
            }
        }
    }
    draw_stars(buf, stars, area);

    let a = fade_alpha(elapsed_ms, LOGO_MS);
    let color = lerp_color(a, 255, 215, 0); // gold/yellow
    let logo_para = Paragraph::new(LOGO_ART)
        .style(Style::default().fg(color).bg(Color::Black))
        .alignment(Alignment::Center);
    let logo_area = centered_sub(area, 70, 7);

    // Write logo into buffer manually so it layers over the stars
    let tmp_buf = {
        let mut b = ratatui::buffer::Buffer::empty(logo_area);
        logo_para.render(logo_area, &mut b);
        b
    };
    for y in 0..logo_area.height {
        for x in 0..logo_area.width {
            let src_pos = Position::new(x, y);
            let dst_pos = Position::new(logo_area.x + x, logo_area.y + y);
            if let (Some(src), Some(dst)) = (tmp_buf.cell(src_pos), buf.cell_mut(dst_pos)) {
                if src.symbol() != " " {
                    dst.set_symbol(src.symbol()).set_fg(src.fg).set_bg(src.bg);
                }
            }
        }
    }
}

fn render_crawl(frame: &mut Frame, stars: &[Star], lines: &[String], scroll: f32) {
    let area = frame.area();
    let buf = frame.buffer_mut();

    // Clear
    for y in 0..area.height {
        for x in 0..area.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_char(' ').set_fg(Color::Black).set_bg(Color::Black);
            }
        }
    }

    draw_stars(buf, stars, area);

    let w = area.width as usize;
    let h = (area.height as usize).saturating_sub(1); // bottom row = progress bar
    let max_w = 75_usize;

    for screen_row in 0..h {
        let depth_idx = h.saturating_sub(1 + screen_row);
        let depth = depth_idx as f32 / (h as f32 - 1.0).max(1.0);

        let line_idx = scroll as i64 - depth_idx as i64;
        if line_idx < 0 || line_idx as usize >= lines.len() {
            continue;
        }

        let perspective = (1.0_f32 - depth).powf(1.5);
        let display_w = ((perspective * max_w as f32) as usize).max(1);

        let chars: Vec<char> = lines[line_idx as usize].chars().collect();
        let clipped: String = if chars.len() > display_w {
            // Center-clip: trim equally from both sides
            let skip = (chars.len() - display_w) / 2;
            chars[skip..skip + display_w].iter().collect()
        } else {
            chars.iter().collect()
        };
        let clipped_len = clipped.chars().count();

        // Two-level centering:
        //   1. Center the perspective band in the terminal
        //   2. Center the actual text within that band
        let band_left = (w.saturating_sub(display_w)) / 2;
        let text_left = band_left + (display_w.saturating_sub(clipped_len)) / 2;

        let brightness = 1.0 - depth;
        let color = Color::Rgb(
            (255.0 * brightness) as u8,
            (200.0 * brightness) as u8,
            0,
        );

        if text_left < w {
            buf.set_string(
                text_left as u16,
                screen_row as u16,
                &clipped,
                Style::default().fg(color).bg(Color::Black),
            );
        }
    }

    // Progress bar
    let total = lines.len() as f32;
    let progress = if total > 0.0 { (scroll / total).clamp(0.0, 1.0) } else { 0.0 };
    let bar_row = area.height - 1;
    let filled = (area.width as f32 * progress) as u16;
    for col in 0..area.width {
        if let Some(cell) = buf.cell_mut(Position::new(col, bar_row)) {
            if col < filled {
                cell.set_char('▓').set_fg(Color::Rgb(230, 180, 0)).set_bg(Color::Black);
            } else {
                cell.set_char('░').set_fg(Color::DarkGray).set_bg(Color::Black);
            }
        }
    }
}

fn run(args: &Args, lines: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let size = terminal.size()?;

    let mut state = if args.skip_intro {
        AppState::Crawl {
            stars: make_stars(size.height, size.width),
            scroll: 0.0,
            speed: args.speed,
            paused: false,
        }
    } else {
        AppState::Prologue { entered: Instant::now() }
    };

    let mut last_frame = Instant::now();

    loop {
        // -- input
        if event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char(' ') => {
                            if let AppState::Crawl { paused, .. } = &mut state {
                                *paused = !*paused;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('=') => {
                            if let AppState::Crawl { speed, .. } = &mut state {
                                *speed = (*speed + SPEED_STEP).min(MAX_SPEED);
                            }
                        }
                        KeyCode::Down | KeyCode::Char('-') => {
                            if let AppState::Crawl { speed, .. } = &mut state {
                                *speed = (*speed - SPEED_STEP).max(0.0);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let now = Instant::now();
        let delta = now.duration_since(last_frame);
        last_frame = now;

        terminal.draw(|frame| match &mut state {
            AppState::Prologue { entered } => {
                render_prologue(frame, entered.elapsed().as_millis() as f32);
            }
            AppState::Logo { entered, stars } => {
                render_logo(frame, stars, entered.elapsed().as_millis() as f32);
            }
            AppState::Crawl { stars, scroll, speed, paused } => {
                if !*paused {
                    *scroll += *speed * delta.as_secs_f32();
                }
                render_crawl(frame, stars, lines, *scroll);
            }
            AppState::Done => {}
        })?;

        // -- state transitions
        let size = terminal.size()?;
        match &state {
            AppState::Prologue { entered } => {
                if entered.elapsed().as_millis() as f32 >= PROLOGUE_MS {
                    state = AppState::Logo {
                        entered: Instant::now(),
                        stars: make_stars(size.height, size.width),
                    };
                }
            }
            AppState::Logo { entered, .. } => {
                if entered.elapsed().as_millis() as f32 >= LOGO_MS {
                    state = AppState::Crawl {
                        stars: make_stars(size.height, size.width),
                        scroll: 0.0,
                        speed: args.speed,
                        paused: false,
                    };
                }
            }
            AppState::Crawl { scroll, .. } => {
                if *scroll as usize >= lines.len() + size.height as usize {
                    state = AppState::Done;
                }
            }
            AppState::Done => break,
        }

        // -- frame rate cap
        let elapsed = last_frame.elapsed();
        let target = Duration::from_millis(FRAME_MS);
        if elapsed < target {
            thread::sleep(target - elapsed);
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    if args.speed <= 0.0 {
        eprintln!("Error: --speed must be positive");
        std::process::exit(1);
    }

    let raw = match std::fs::read_to_string(&args.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {:?}: {e}", args.file);
            std::process::exit(1);
        }
    };

    let mut lines = text::process_lines(&raw, args.width, args.wrap);
    if !args.no_header {
        let header = args
            .file
            .file_name()
            .map(|n| n.to_string_lossy().to_uppercase().to_string())
            .unwrap_or_default();
        lines.insert(0, String::new());
        lines.insert(0, header);
        lines.insert(0, String::new());
    }

    enable_raw_mode().expect("failed to enable raw mode");
    execute!(stdout(), EnterAlternateScreen, cursor::Hide)
        .expect("failed to enter alternate screen");

    let result = run(&args, &lines);

    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen, cursor::Show);

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

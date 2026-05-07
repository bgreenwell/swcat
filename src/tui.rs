use std::{
    io::stdout,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

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
    widgets::Paragraph,
    Frame, Terminal,
};

use crate::text;

const PROLOGUE_MS: f32 = 4000.0;
const LOGO_MS: f32 = 6000.0;
const SPEED_STEP: f32 = 0.5;
const MAX_SPEED: f32 = 20.0;
const STAR_COUNT: usize = 180;
const FRAME_MS: u64 = 33; // ~30 fps
const MAX_W: usize = 75;
const TAPER: usize = 1; // rows between each 1-char convergence per side

const LOGO_ART: &str = r#" _____ __        __   ____     _    _____  
/ ___| \ \      / /  / ___|   / \   |_   _|
\___ \  \ \ /\ / /  | |      / _ \    | |  
 ___) |  \ V  V /   | |___  / ___ \   | |  
|____/    \_/\_/     \____|/_/   \_\  |_|  "#;

const PROLOGUE_TEXT: &str = "A long time ago in a terminal far,\nfar away....";

pub struct TuiArgs {
    pub file: PathBuf,
    pub speed: f32,
    pub skip_intro: bool,
    pub width: Option<usize>,
    pub wrap: bool,
    pub no_header: bool,
    pub border: bool,
    pub left: bool,
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

fn render_prologue(frame: &mut Frame, elapsed_ms: f32) {
    let area = frame.area();
    let a = fade_alpha(elapsed_ms, PROLOGUE_MS);
    let color = lerp_color(a, 0, 204, 255); // cyan

    let text_w = PROLOGUE_TEXT.lines().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    let text_h = PROLOGUE_TEXT.lines().count() as u16;
    let tx = area.x + area.width.saturating_sub(text_w) / 2;
    let ty = area.y + area.height.saturating_sub(text_h) / 2;
    let text_area = Rect::new(tx, ty, text_w.min(area.width), text_h.min(area.height));

    let para = Paragraph::new(PROLOGUE_TEXT)
        .style(Style::default().fg(color).bg(Color::Black))
        .alignment(Alignment::Left);

    frame.render_widget(Paragraph::new("").style(Style::default().bg(Color::Black)), area);
    frame.render_widget(para, text_area);
}

fn render_logo(frame: &mut Frame, stars: &[Star], elapsed_ms: f32) {
    let area = frame.area();
    let buf = frame.buffer_mut();

    for y in 0..area.height {
        for x in 0..area.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_char(' ').set_fg(Color::Black).set_bg(Color::Black);
            }
        }
    }
    draw_stars(buf, stars, area);

    let a = fade_alpha(elapsed_ms, LOGO_MS);
    let color = lerp_color(a, 255, 215, 0); // gold

    let logo_w = LOGO_ART.lines().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    let logo_h = LOGO_ART.lines().count() as u16;
    let lx = area.x + area.width.saturating_sub(logo_w) / 2;
    let ly = area.y + area.height.saturating_sub(logo_h) / 2;
    let logo_area = Rect::new(lx, ly, logo_w.min(area.width), logo_h.min(area.height));

    let logo_para = Paragraph::new(LOGO_ART)
        .style(Style::default().fg(color).bg(Color::Black))
        .alignment(Alignment::Left);

    let tmp_buf = {
        let mut b = ratatui::buffer::Buffer::empty(logo_area);
        logo_para.render(logo_area, &mut b);
        b
    };
    for row in 0..logo_area.height {
        for col in 0..logo_area.width {
            let pos = Position::new(logo_area.x + col, logo_area.y + row);
            if let (Some(src), Some(dst)) = (tmp_buf.cell(pos), buf.cell_mut(pos)) {
                if src.symbol() != " " {
                    dst.set_symbol(src.symbol()).set_fg(src.fg).set_bg(src.bg);
                }
            }
        }
    }
}

fn render_crawl(frame: &mut Frame, stars: &[Star], lines: &[String], scroll: f32, border: bool, no_header: bool, left: bool) {
    let area = frame.area();
    let buf = frame.buffer_mut();

    for y in 0..area.height {
        for x in 0..area.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_char(' ').set_fg(Color::Black).set_bg(Color::Black);
            }
        }
    }
    draw_stars(buf, stars, area);

    let w = area.width as usize;
    let h = (area.height as usize).saturating_sub(1);

    let band_w = MAX_W + 2;
    let band_left = (w.saturating_sub(band_w)) / 2;

    for screen_row in 0..h {
        let depth_idx = h.saturating_sub(1 + screen_row);
        let shrink = depth_idx / TAPER;

        let slash_col = band_left + shrink;
        let backslash_col = band_left + band_w - 1 - shrink;

        if slash_col >= backslash_col {
            continue;
        }

        let inner_w = backslash_col - slash_col - 1;
        let brightness = 1.0 - (depth_idx as f32 / h as f32);
        let color = Color::Rgb(
            (255.0 * brightness) as u8,
            (200.0 * brightness) as u8,
            0,
        );

        if border {
            if slash_col < w {
                if let Some(cell) = buf.cell_mut(Position::new(slash_col as u16, screen_row as u16)) {
                    cell.set_char('/').set_fg(color).set_bg(Color::Black);
                }
            }
            if backslash_col < w {
                if let Some(cell) = buf.cell_mut(Position::new(backslash_col as u16, screen_row as u16)) {
                    cell.set_char('\\').set_fg(color).set_bg(Color::Black);
                }
            }
        }

        let line_idx = scroll as i64 - depth_idx as i64;
        if line_idx >= 0 && (line_idx as usize) < lines.len() {
            let chars: Vec<char> = lines[line_idx as usize].chars().collect();
            
            let is_header = !no_header && line_idx == 1;
            let should_center = is_header || !left;
            
            let text_col = if should_center {
                slash_col + 1 + (inner_w.saturating_sub(chars.len()) / 2)
            } else {
                slash_col + 1
            };

            let visible: String = chars.iter().take(inner_w.saturating_sub(text_col.saturating_sub(slash_col + 1))).collect();
            if text_col < w && !visible.is_empty() {
                buf.set_string(
                    text_col as u16,
                    screen_row as u16,
                    &visible,
                    Style::default().fg(color).bg(Color::Black),
                );
            }
        }
    }

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

pub fn run(args: TuiArgs) -> Result<(), Box<dyn std::error::Error>> {
    let raw = std::fs::read_to_string(&args.file)?;
    
    let wrap_w = args.width.unwrap_or(MAX_W);
    let mut lines = text::process_lines(&raw, Some(wrap_w), true);
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

    let result = loop {
        if event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
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
                render_crawl(frame, stars, &lines, *scroll, args.border, args.no_header, args.left);
            }
            AppState::Done => {}
        })?;

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
            AppState::Done => break Ok(()),
        }

        let elapsed = last_frame.elapsed();
        let target = Duration::from_millis(FRAME_MS);
        if elapsed < target {
            thread::sleep(target - elapsed);
        }
    };

    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen, cursor::Show);

    result
}

use macroquad::prelude::*;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use std::path::PathBuf;
use crate::text;

pub const FONT_BYTES: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    color = color0;
    uv = texcoord;
}
";

const FRAGMENT_SHADER: &str = "#version 100
precision mediump float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
uniform vec2 resolution;
uniform float thickness;

void main() {
    float a = texture2D(Texture, uv).a;
    float max_a = 0.0;
    vec2 off = 1.0 / resolution;

    // Wider sampling for a thicker bubble look
    for (float i = -2.0; i <= 2.0; i += 1.0) {
        for (float j = -2.0; j <= 2.0; j += 1.0) {
            float s = texture2D(Texture, uv + vec2(i, j) * off * thickness).a;
            if (s > max_a) max_a = s;
        }
    }

    // Inside is transparent (a=1), outside is yellow (max_a=1, a=0)
    float outline = max_a - a;
    gl_FragColor = vec4(1.0, 1.0, 0.0, outline * color.a);
}
";

// -- Display ------------------------------------------------------------------
pub const BASE_WINDOW_W: i32 = 1000; // golden-ratio base width
pub const BASE_WINDOW_H: i32 = 618;  // BASE_WINDOW_W / phi
const FONT_SIZE: u16 = 40;
const HEADER_FONT_SIZE: u16 = 70;
const LINE_HEIGHT: f32 = 55.0;

// -- Texture / chunk geometry -------------------------------------------------
const TEXTURE_PADDING: u32 = 100;
const MIN_TEXTURE_WIDTH: u32 = 1024;
const LINES_PER_CHUNK: usize = 60;
const CHUNK_TEXTURE_PAD: f32 = 150.0;
const PLANE_BASE_WIDTH: f32 = 15.0;

// -- Logo ---------------------------------------------------------------------
const LOGO_TEXTURE_W: u32 = 2048;
const LOGO_TEXTURE_H: u32 = 1024;
const LOGO_FONT_SIZE: u16 = 650;
const LOGO_DURATION: f32 = 6.0;
const LOGO_BORDER_THICKNESS: f32 = 4.0;

// -- Timing -------------------------------------------------------------------
const PROLOGUE_DURATION: f32 = 4.0;

// -- Stars --------------------------------------------------------------------
const STAR_COUNT: usize = 200;
const STAR_SIZE_MIN: f32 = 0.5;
const STAR_SIZE_MAX: f32 = 2.0;
const STAR_BRIGHTNESS_MIN: f32 = 0.5;
const STAR_BRIGHTNESS_MAX: f32 = 1.0;

// -- Scroll -------------------------------------------------------------------
const SCROLL_SCALE: f32 = 60.0;
const SCROLL_START_OFFSET: f32 = 10.5;
const WHEEL_SENSITIVITY: f32 = 100.0;
const SPEED_STEP: f32 = 10.0;
const MAX_SPEED: f32 = 500.0;

// -- Crawl UI -----------------------------------------------------------------
const PROGRESS_BAR_H: f32 = 3.0;

struct Star {
    x: f32,
    y: f32,
    size: f32,
    brightness: f32,
}

#[derive(PartialEq)]
enum State {
    Prologue,
    Logo,
    Crawl,
    Done,
}

pub struct GuiArgs {
    pub file: PathBuf,
    pub speed: f32,
    pub skip_intro: bool,
    pub left: bool,
    pub width: Option<usize>,
    pub wrap: bool,
    pub no_header: bool,
}

fn draw_stars(stars: &[Star]) {
    for star in stars {
        draw_circle(
            star.x / BASE_WINDOW_W as f32 * screen_width(),
            star.y / BASE_WINDOW_H as f32 * screen_height(),
            star.size,
            Color::new(star.brightness, star.brightness, star.brightness, 1.0),
        );
    }
}

pub async fn run(args: GuiArgs) {
    let content = match std::fs::read_to_string(&args.file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading '{}': {}", args.file.display(), e);
            return;
        }
    };

    let stem = args
        .file
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_uppercase();

    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            uniforms: vec![
                UniformDesc::new("resolution", UniformType::Float2),
                UniformDesc::new("thickness", UniformType::Float1),
            ],
            ..Default::default()
        },
    )
    .expect("Failed to load hollow text material");

    let font = load_ttf_font_from_bytes(FONT_BYTES).expect("Failed to load font");

    let stars: Vec<Star> = (0..STAR_COUNT)
        .map(|_| Star {
            x: rand::gen_range(0.0f32, BASE_WINDOW_W as f32),
            y: rand::gen_range(0.0f32, BASE_WINDOW_H as f32),
            size: rand::gen_range(STAR_SIZE_MIN, STAR_SIZE_MAX),
            brightness: rand::gen_range(STAR_BRIGHTNESS_MIN, STAR_BRIGHTNESS_MAX),
        })
        .collect();

    let mut lines = text::process_lines(&content, args.width, args.wrap);

    if !args.no_header {
        let mut headed = vec![stem.clone(), String::new()];
        headed.extend(lines);
        lines = headed;
    }

    let mut max_text_width = 0.0f32;
    for (i, line) in lines.iter().enumerate() {
        let size = if i == 0 && !args.no_header { HEADER_FONT_SIZE } else { FONT_SIZE };
        let dim = measure_text(line, Some(&font), size, 1.0);
        if dim.width > max_text_width {
            max_text_width = dim.width;
        }
    }

    let texture_width = (max_text_width as u32 + TEXTURE_PADDING).max(MIN_TEXTURE_WIDTH);
    let plane_width = PLANE_BASE_WIDTH * (texture_width as f32 / MIN_TEXTURE_WIDTH as f32);

    let mut chunks: Vec<(RenderTarget, f32)> = Vec::new();
    let mut total_plane_height = 0.0f32;

    for (chunk_idx, lines_chunk) in lines.chunks(LINES_PER_CHUNK).enumerate() {
        let chunk_h = (lines_chunk.len() as f32 * LINE_HEIGHT + CHUNK_TEXTURE_PAD) as u32;
        let target = render_target(texture_width, chunk_h);
        target.texture.set_filter(FilterMode::Linear);

        set_camera(&Camera2D {
            render_target: Some(target.clone()),
            zoom: vec2(2.0 / texture_width as f32, 2.0 / chunk_h as f32),
            target: vec2(texture_width as f32 / 2.0, chunk_h as f32 / 2.0),
            ..Default::default()
        });

        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        for (i, line) in lines_chunk.iter().enumerate() {
            let is_header = !args.no_header && chunk_idx == 0 && i == 0;
            let font_size = if is_header { HEADER_FONT_SIZE } else { FONT_SIZE };

            let x = if args.left && !is_header {
                (texture_width as f32 - max_text_width) / 2.0
            } else {
                let dim = measure_text(line, Some(&font), font_size, 1.0);
                (texture_width as f32 - dim.width) / 2.0
            };
            draw_text_ex(line, x, i as f32 * LINE_HEIGHT + 60.0, TextParams {
                font: Some(&font), font_size, color: YELLOW, ..Default::default()
            });
        }

        let chunk_plane_height = plane_width * (chunk_h as f32 / texture_width as f32);
        total_plane_height += chunk_plane_height;
        chunks.push((target, chunk_plane_height));
    }
    set_default_camera();

    let logo_target = render_target(LOGO_TEXTURE_W, LOGO_TEXTURE_H);
    logo_target.texture.set_filter(FilterMode::Linear);
    let mut logo_rendered = false;

    let mut state = if args.skip_intro { State::Crawl } else { State::Prologue };
    let mut state_time = 0.0f32;
    let mut scroll_y = -SCROLL_SCALE * SCROLL_START_OFFSET;
    let mut scroll_speed = args.speed;
    let mut paused = false;

    loop {
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
            break;
        }

        let delta = get_frame_time();
        state_time += delta;

        clear_background(BLACK);

        match state {
            State::Prologue => {
                let text = "A long time ago in a terminal far,\nfar away....";
                let color = Color::new(0.0, 0.8, 1.0, 1.0);
                let p_font_size: u16 = 30;
                let line_spacing = 40.0;

                let p_lines: Vec<&str> = text.lines().collect();
                let block_w = p_lines.iter()
                    .map(|l| measure_text(l, Some(&font), p_font_size, 1.0).width)
                    .fold(0.0f32, f32::max);
                let x = (screen_width() - block_w) / 2.0;
                let total_h = p_lines.len() as f32 * line_spacing;
                let mut y = (screen_height() - total_h) / 2.0;
                for line in &p_lines {
                    draw_text_ex(line, x, y, TextParams {
                        font: Some(&font), font_size: p_font_size, color, ..Default::default()
                    });
                    y += line_spacing;
                }

                if state_time > PROLOGUE_DURATION {
                    state = State::Logo;
                    state_time = 0.0;
                }
            }

            State::Logo => {
                draw_stars(&stars);

                if !logo_rendered {
                    let logo_text = "SWCAT";
                    let dim = measure_text(logo_text, Some(&font), LOGO_FONT_SIZE, 1.0);
                    set_camera(&Camera2D {
                        render_target: Some(logo_target.clone()),
                        zoom: vec2(2.0 / LOGO_TEXTURE_W as f32, -2.0 / LOGO_TEXTURE_H as f32),
                        target: vec2(LOGO_TEXTURE_W as f32 / 2.0, LOGO_TEXTURE_H as f32 / 2.0),
                        ..Default::default()
                    });
                    clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
                    draw_text_ex(
                        logo_text,
                        (LOGO_TEXTURE_W as f32 - dim.width) / 2.0,
                        LOGO_TEXTURE_H as f32 / 2.0 + dim.height / 4.0,
                        TextParams { font: Some(&font), font_size: LOGO_FONT_SIZE, color: WHITE, ..Default::default() },
                    );
                    set_default_camera();
                    logo_rendered = true;
                }

                let t = state_time / LOGO_DURATION;
                let scale = (1.0 - t).powf(2.5).max(0.0);

                if scale > 0.005 {
                    let screen_aspect = screen_width() / screen_height();
                    let target_aspect = LOGO_TEXTURE_W as f32 / LOGO_TEXTURE_H as f32;

                    let base_draw_w = if screen_aspect > target_aspect {
                        screen_height() * target_aspect
                    } else {
                        screen_width()
                    };
                    let base_draw_h = base_draw_w / target_aspect;

                    gl_use_material(&material);
                    material.set_uniform("resolution", vec2(LOGO_TEXTURE_W as f32, LOGO_TEXTURE_H as f32));
                    material.set_uniform("thickness", LOGO_BORDER_THICKNESS);

                    draw_texture_ex(
                        &logo_target.texture,
                        (screen_width() - base_draw_w * scale) / 2.0,
                        (screen_height() - base_draw_h * scale) / 2.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(base_draw_w * scale, base_draw_h * scale)),
                            flip_y: true,
                            ..Default::default()
                        },
                    );
                    gl_use_default_material();
                }

                if state_time > LOGO_DURATION {
                    state = State::Crawl;
                    state_time = 0.0;
                }
            }

            State::Crawl => {
                draw_stars(&stars);

                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Equal) {
                    scroll_speed = (scroll_speed + SPEED_STEP).min(MAX_SPEED);
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::Minus) {
                    scroll_speed = (scroll_speed - SPEED_STEP).max(0.0);
                }
                if is_key_pressed(KeyCode::Space) {
                    paused = !paused;
                }

                let (_, wheel_y) = mouse_wheel();
                if wheel_y != 0.0 {
                    scroll_y -= wheel_y * WHEEL_SENSITIVITY;
                } else if !paused {
                    scroll_y += scroll_speed * delta;
                }

                let y_offset = -scroll_y / SCROLL_SCALE;

                if y_offset + total_plane_height < 0.0 {
                    state = State::Done;
                }

                set_camera(&Camera3D {
                    position: vec3(0.0, 5.0, 10.0),
                    up: vec3(0.0, 1.0, 0.0),
                    target: vec3(0.0, 0.0, 0.0),
                    ..Default::default()
                });

                let mut current_z = 0.0f32;
                for (chunk_target, chunk_plane_height) in &chunks {
                    let z_far = y_offset + current_z;
                    let z_near = z_far + chunk_plane_height;
                    draw_mesh(&Mesh {
                        vertices: vec![
                            Vertex { position: vec3(-plane_width / 2.0, 0.0, z_far),  uv: vec2(0.0, 0.0), normal: vec4(0.0, 1.0, 0.0, 0.0), color: WHITE.into() },
                            Vertex { position: vec3( plane_width / 2.0, 0.0, z_far),  uv: vec2(1.0, 0.0), normal: vec4(0.0, 1.0, 0.0, 0.0), color: WHITE.into() },
                            Vertex { position: vec3( plane_width / 2.0, 0.0, z_near), uv: vec2(1.0, 1.0), normal: vec4(0.0, 1.0, 0.0, 0.0), color: WHITE.into() },
                            Vertex { position: vec3(-plane_width / 2.0, 0.0, z_near), uv: vec2(0.0, 1.0), normal: vec4(0.0, 1.0, 0.0, 0.0), color: WHITE.into() },
                        ],
                        indices: vec![0, 1, 2, 0, 2, 3],
                        texture: Some(chunk_target.texture.clone()),
                    });
                    current_z += chunk_plane_height;
                }

                set_default_camera();

                let progress = ((SCROLL_START_OFFSET - y_offset)
                    / (SCROLL_START_OFFSET + total_plane_height))
                    .clamp(0.0, 1.0);
                let bar_y = screen_height() - PROGRESS_BAR_H;
                draw_rectangle(0.0, bar_y, screen_width(), PROGRESS_BAR_H,
                    Color::new(1.0, 1.0, 1.0, 0.15));
                draw_rectangle(0.0, bar_y, screen_width() * progress, PROGRESS_BAR_H,
                    Color::new(0.9, 0.8, 0.1, 0.8));
            }

            State::Done => break,
        }

        next_frame().await
    }
}

mod camera3d;
mod geometry;
mod state2d;
mod state3d;

use self::state2d::State2d;
use self::state3d::State3d;

use geng::prelude::*;
use geng_utils::{conversions::Vec2RealConversions, key as key_utils};

#[derive(clap::Parser)]
struct Opts {
    #[clap(flatten)]
    window: geng::CliArgs,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub config: Hot<Config>,
    pub simple3d: Hot<ugli::Program>,
    pub outline_marker: Hot<ugli::Program>,
    pub postprocess: Hot<ugli::Program>,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "ron")]
pub struct Config {
    object_limit: usize,
    spawn_depth_min: f32,
    spawn_depth_max: f32,
    scale_min: f32,
    scale_max: f32,
    speed: f32,
    rotation_speed_degrees: f32,
    background_color: Rgba<f32>,
    object_colors: Vec<Rgba<f32>>,
}

enum Mode {
    Mode2d,
    Mode3d,
}

pub struct State {
    geng: Geng,
    assets: Rc<Assets>,
    cursor_pos: vec2<f32>,
    paused: bool,
    mode: Mode,
    state2d: State2d,
    state3d: State3d,
    button2d: Aabb2<f32>,
    button3d: Aabb2<f32>,
}

impl State {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        Self {
            paused: false,
            mode: Mode::Mode2d,
            cursor_pos: vec2::ZERO,
            state2d: State2d::new(geng.clone(), assets.clone()),
            state3d: State3d::new(geng.clone(), assets.clone()),
            button2d: Aabb2::ZERO,
            button3d: Aabb2::ZERO,
            geng,
            assets,
        }
    }
}

impl geng::State for State {
    fn update(&mut self, delta_time: f64) {
        if !self.paused {
            match self.mode {
                Mode::Mode2d => {
                    self.state2d.update(delta_time);
                }
                Mode::Mode3d => {
                    self.state3d.update(delta_time);
                }
            }
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::CursorMove { position } = event {
            self.cursor_pos = position.as_f32();
        }

        if key_utils::is_event_press(&event, [geng::Key::P]) {
            self.paused = !self.paused;
        }

        if key_utils::is_event_press(&event, [geng::MouseButton::Left]) {
            if self.button2d.contains(self.cursor_pos) {
                self.mode = Mode::Mode2d;
            } else if self.button3d.contains(self.cursor_pos) {
                self.mode = Mode::Mode3d;
            }
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(
            framebuffer,
            Some(self.assets.config.get().background_color),
            Some(1.0),
            None,
        );

        // State
        match self.mode {
            Mode::Mode2d => {
                self.state2d.draw(framebuffer);
            }
            Mode::Mode3d => {
                self.state3d.draw(framebuffer);
            }
        }

        // UI
        let framebuffer_size = framebuffer.size().as_f32();
        let camera = &geng::PixelPerfectCamera;
        let font_size = framebuffer_size.x.min(framebuffer_size.y) * 0.02;

        let mut draw_button = |text: &str, position: Aabb2<f32>, active: bool| {
            let color = if active {
                Rgba::try_from("#aaa").unwrap()
            } else {
                Rgba::try_from("#555").unwrap()
            };
            self.geng
                .draw2d()
                .draw2d(framebuffer, camera, &draw2d::Quad::new(position, color));

            let color = if position.contains(self.cursor_pos) {
                if key_utils::is_key_pressed(self.geng.window(), [geng::MouseButton::Left]) {
                    // Pressed
                    Rgba::try_from("#111").unwrap()
                } else {
                    // Hovered
                    Rgba::try_from("#333").unwrap()
                }
            } else {
                Rgba::try_from("#222").unwrap()
            };
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(position.extend_uniform(-font_size * 0.2), color),
            );

            self.geng.default_font().draw(
                framebuffer,
                camera,
                text,
                vec2::splat(geng::TextAlign::CENTER),
                mat3::translate(
                    geng_utils::layout::aabb_pos(
                        position.extend_symmetric(vec2(-font_size, 0.0)),
                        vec2(0.5, 0.5),
                    ) + vec2(0.0, -font_size / 4.0),
                ) * mat3::scale_uniform(font_size),
                Rgba::WHITE,
            );
        };

        let button_size = vec2(4.0, 2.0) * font_size;
        let button = Aabb2::ZERO
            .extend_positive(button_size)
            .translate(vec2(0.0, -button_size.y));
        let pos = vec2(0.0, framebuffer_size.y) + vec2(1.0, -1.0) * font_size;

        self.button2d = button.translate(pos);
        draw_button("2D", self.button2d, matches!(self.mode, Mode::Mode2d));

        self.button3d = button.translate(pos - vec2(0.0, button_size.y + font_size));
        draw_button("3D", self.button3d, matches!(self.mode, Mode::Mode3d));
    }
}

fn main() {
    logger::init();

    let opts: Opts = clap::Parser::parse();

    let mut context = geng::ContextOptions::default();
    context.with_cli(&opts.window);
    Geng::run_with(&context, |geng| async move {
        let manager = geng.asset_manager();
        let assets: Rc<Assets> = geng::asset::Load::load(manager, &run_dir().join("assets"), &())
            .await
            .expect("failed to load assets");
        geng.run_state(State::new(geng.clone(), assets)).await
    })
}

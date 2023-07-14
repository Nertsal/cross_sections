mod camera3d;
mod geometry;
mod state2d;
mod state3d;

use self::state2d::State2d;
use self::state3d::State3d;

use geng::prelude::*;

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
    paused: bool,
    mode: Mode,
    state2d: State2d,
    state3d: State3d,
}

impl State {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        Self {
            paused: false,
            mode: Mode::Mode2d,
            state2d: State2d::new(geng.clone(), assets.clone()),
            state3d: State3d::new(geng.clone(), assets.clone()),
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
        if geng_utils::key::is_event_press(&event, [geng::Key::P]) {
            self.paused = !self.paused;
        }

        if geng_utils::key::is_event_press(&event, [geng::Key::Space]) {
            self.mode = match self.mode {
                Mode::Mode2d => Mode::Mode3d,
                Mode::Mode3d => Mode::Mode2d,
            };
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(
            framebuffer,
            Some(self.assets.config.get().background_color),
            Some(1.0),
            None,
        );

        match self.mode {
            Mode::Mode2d => {
                self.state2d.draw(framebuffer);
            }
            Mode::Mode3d => {
                self.state3d.draw(framebuffer);
            }
        }
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

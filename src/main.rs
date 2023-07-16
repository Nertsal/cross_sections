mod camera3d;
mod geometry;
mod state2d;
mod state3d;

use self::state2d::State2d;
use self::state3d::State3d;

use geng::prelude::*;
use geng_utils::{bounded::Bounded, conversions::Vec2RealConversions, key as key_utils};

#[derive(clap::Parser)]
struct Opts {
    #[clap(flatten)]
    window: geng::CliArgs,
}

#[derive(geng::asset::Load)]
pub struct Assets {
    pub cut_front: Hot<ugli::Program>,
    pub cut_back: Hot<ugli::Program>,
    pub cross: Hot<ugli::Program>,
    pub simple3d: Hot<ugli::Program>,
    pub outline_marker: Hot<ugli::Program>,
    pub postprocess: Hot<ugli::Program>,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "ron")]
pub struct Config {
    object_limit: Bounded<usize>,
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
    config: Config,
    cursor_pos: vec2<f32>,
    touch_pos: vec2<f32>,
    paused: bool,
    mode: Mode,
    include_3d_in_2d: bool,
    state2d: State2d,
    state3d: State3d,
    drag: Option<Drag>,
    button2d: Aabb2<f32>,
    button3d: Aabb2<f32>,
    button_include3d: Aabb2<f32>,
    slider_object_limit: Aabb2<f32>,
}

enum Drag {
    SliderObjects,
}

impl State {
    pub fn new(geng: Geng, assets: Rc<Assets>, config: Config) -> Self {
        Self {
            paused: false,
            mode: Mode::Mode2d,
            include_3d_in_2d: false,
            cursor_pos: vec2::ZERO,
            touch_pos: vec2::ZERO,
            state2d: State2d::new(geng.clone(), assets.clone()),
            state3d: State3d::new(geng.clone(), assets.clone()),
            drag: None,
            button2d: Aabb2::ZERO,
            button3d: Aabb2::ZERO,
            button_include3d: Aabb2::ZERO,
            slider_object_limit: Aabb2::ZERO,
            geng,
            assets,
            config,
        }
    }

    fn touch_press(&mut self, pos: vec2<f32>) {
        self.touch_pos = pos;
        if self.slider_object_limit.contains(pos) {
            self.drag = Some(Drag::SliderObjects);
        }
    }

    fn touch_move(&mut self, pos: vec2<f32>) {
        if let Some(drag) = &self.drag {
            match drag {
                Drag::SliderObjects => {
                    let t =
                        (pos.x - self.slider_object_limit.min.x) / self.slider_object_limit.width();
                    let mut value = self.config.object_limit.map(|x| x as f32);
                    value.set_ratio(t);
                    self.config.object_limit = value.map(|x| x.floor() as usize);
                }
            }
        }
    }

    fn touch_release(&mut self, pos: vec2<f32>) {
        if (self.touch_pos - pos).len_sqr() < 1.0 {
            // Click
            if self.button2d.contains(pos) {
                self.mode = Mode::Mode2d;
            } else if self.button3d.contains(pos) {
                self.mode = Mode::Mode3d;
            } else if matches!(self.mode, Mode::Mode2d) && self.button_include3d.contains(pos) {
                self.include_3d_in_2d = !self.include_3d_in_2d;
            }
        }
        self.drag = None;
    }

    fn draw_ui(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.draw_mode_ui(framebuffer);
        self.draw_config_ui(framebuffer);
    }

    fn draw_mode_ui(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let framebuffer_size = framebuffer.size().as_f32();
        let camera = &geng::PixelPerfectCamera;
        let font_size = framebuffer_size.x.min(framebuffer_size.y) * 0.02;
        let font_size = font_size.max(20.0);

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
        draw_button("3D -> 2D", self.button2d, matches!(self.mode, Mode::Mode2d));

        self.button3d = button.translate(pos - vec2(0.0, button_size.y + font_size));
        draw_button("4D -> 3D", self.button3d, matches!(self.mode, Mode::Mode3d));

        // Tickbox
        if let Mode::Mode2d = self.mode {
            let tickbox_size = vec2::splat(1.5) * font_size;
            let tickbox = Aabb2::ZERO.extend_symmetric(tickbox_size / 2.0);
            let pos = geng_utils::layout::aabb_pos(self.button2d, vec2(1.0, 0.5));

            self.button_include3d = tickbox
                .translate(pos)
                .translate(vec2(font_size + tickbox.width() / 2.0, 0.0));
            let position = self.button_include3d;

            // Outline
            let color = if self.include_3d_in_2d {
                Rgba::try_from("#aaa").unwrap()
            } else {
                Rgba::try_from("#555").unwrap()
            };
            self.geng
                .draw2d()
                .draw2d(framebuffer, camera, &draw2d::Quad::new(position, color));

            // Fill
            let color = if position.contains(self.cursor_pos) {
                // Hovered
                Rgba::try_from("#333").unwrap()
            } else {
                Rgba::try_from("#222").unwrap()
            };
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Quad::new(position.extend_uniform(-font_size * 0.2), color),
            );

            // Text
            let font_size = font_size * 0.8;
            self.geng.default_font().draw(
                framebuffer,
                camera,
                "3d",
                vec2::splat(geng::TextAlign::CENTER),
                mat3::translate(position.center() + vec2(0.0, -font_size / 4.0))
                    * mat3::scale_uniform(font_size),
                Rgba::WHITE,
            );
        }
    }

    fn draw_config_ui(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let framebuffer_size = framebuffer.size().as_f32();
        let camera = &geng::PixelPerfectCamera;
        let font_size = framebuffer_size.x.min(framebuffer_size.y) * 0.02;
        let font_size = font_size.max(20.0);

        let mut draw_slider = |text: &str, position: Aabb2<f32>, value: f32| {
            let color = Rgba::try_from("#777").unwrap();
            self.geng
                .draw2d()
                .draw2d(framebuffer, camera, &draw2d::Quad::new(position, color));

            // Slider button
            let color = if position.contains(self.cursor_pos) {
                if key_utils::is_key_pressed(self.geng.window(), [geng::MouseButton::Left]) {
                    // Pressed
                    Rgba::try_from("#333").unwrap()
                } else {
                    // Hovered
                    Rgba::try_from("#555").unwrap()
                }
            } else {
                Rgba::try_from("#444").unwrap()
            };
            let pos = geng_utils::layout::aabb_pos(position, vec2(value, 0.5));
            self.geng.draw2d().draw2d(
                framebuffer,
                camera,
                &draw2d::Ellipse::circle(pos, font_size * 0.5, color),
            );

            // Text
            self.geng.default_font().draw(
                framebuffer,
                camera,
                text,
                vec2::splat(geng::TextAlign::RIGHT),
                mat3::translate(
                    geng_utils::layout::aabb_pos(position, vec2(0.0, 0.5))
                        + vec2(-font_size * 0.5, -font_size / 4.0),
                ) * mat3::scale_uniform(font_size),
                Rgba::WHITE,
            );
        };

        let slider_size = vec2(5.0, 0.3) * font_size;
        let slider = Aabb2::ZERO
            .extend_positive(slider_size)
            .translate(vec2(-slider_size.x, -slider_size.y));

        let pos = framebuffer_size - vec2(1.0, 1.0) * font_size;
        self.slider_object_limit = slider.translate(pos);
        draw_slider(
            &format!("Objects {:2}", self.config.object_limit.value()),
            self.slider_object_limit,
            self.config.object_limit.map(|x| x as f32).get_ratio(),
        );
    }
}

impl geng::State for State {
    fn update(&mut self, delta_time: f64) {
        if !self.paused {
            match self.mode {
                Mode::Mode2d => {
                    self.state2d.update(&self.config, delta_time);
                }
                Mode::Mode3d => {
                    self.state3d.update(&self.config, delta_time);
                }
            }
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        let pass_to_state = true;

        if let geng::Event::CursorMove { position } = event {
            self.cursor_pos = position.as_f32();
            self.touch_move(self.cursor_pos);
        }
        if let geng::Event::TouchMove(touch) = &event {
            self.touch_move(touch.position.as_f32());
        }

        if key_utils::is_event_press(&event, [geng::Key::P]) {
            self.paused = !self.paused;
        }

        // TODO: multitouch
        if let geng::Event::TouchStart(touch) = &event {
            self.touch_press(touch.position.as_f32());
        }
        if key_utils::is_event_press(&event, [geng::MouseButton::Left]) {
            self.touch_press(self.cursor_pos);
        }

        if let geng::Event::TouchEnd(touch) = &event {
            self.touch_release(touch.position.as_f32());
        }
        if key_utils::is_event_release(&event, [geng::MouseButton::Left]) {
            self.touch_release(self.cursor_pos);
        }

        if pass_to_state {
            match self.mode {
                Mode::Mode2d => self.state2d.handle_event(&event),
                Mode::Mode3d => self.state3d.handle_event(&event),
            }
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(
            framebuffer,
            Some(self.config.background_color),
            Some(1.0),
            None,
        );

        // State
        match self.mode {
            Mode::Mode2d => {
                self.state2d
                    .draw(&self.config, self.include_3d_in_2d, framebuffer);
            }
            Mode::Mode3d => {
                self.state3d.draw(&self.config, framebuffer);
            }
        }

        self.draw_ui(framebuffer);
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
        let config: Config =
            geng::asset::Load::load(manager, &run_dir().join("assets").join("config.ron"), &())
                .await
                .expect("failed to load config");
        geng.run_state(State::new(geng.clone(), assets, config))
            .await
    })
}

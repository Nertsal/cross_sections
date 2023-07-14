use crate::Assets;

use geng::prelude::*;

pub struct State3d {}

impl State3d {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        Self {}
    }
}

impl geng::State for State3d {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
    }
}

use wasm_bindgen::prelude::wasm_bindgen;

use asteroids::Controls;

pub mod render;
use render::PathList;

#[wasm_bindgen]
pub struct App(asteroids::App);

#[wasm_bindgen]
impl App {
    pub fn new() -> Self {
        App(asteroids::App::new())
    }

    pub fn step(&mut self, dt: f64, input: u32) -> () {
        if dt <= 0.0 {
            return ();
        }
        self.0.step(dt, Controls::new(input))
    }

    pub fn render(&self) -> PathList {
        let mut list = PathList::new();
        render::player(&self.0.player, &mut list);
        render::asteroids(&self.0.asteroids, &mut list);
        render::blasts(&self.0.blasts, &mut list);
        list
    }
}

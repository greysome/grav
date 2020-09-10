use ggez::nalgebra::Point2;
use ggez::graphics::Font;

use crate::body::Body;
use crate::ui::UiWrapper;

#[derive(PartialEq)]
pub enum GameMode {
    Drag, Add
}

pub struct GameState {
    pub size: (f32, f32), // (width, height)
    pub font: Font,

    pub origin: Point2<f32>, // Position of center on global xy-plane
    pub scale: f32, // 1 pixel corresponds to `scale` units on global xy-plane
    pub bodies: Vec<Body>,

    pub dt: f32, // Number of seconds that pass in a step
    pub paused: bool,
    pub mode: GameMode,
}

pub struct GameInstance {
    // Data is organised this way so `state` and `ui_wrapper` can be
    // borrowed at the same time
    pub state: GameState,
    pub ui_wrapper: UiWrapper
}

use ggez::nalgebra::Point2;

use crate::body::Body;
use crate::ui::UiWrapper;

#[derive(PartialEq)]
pub enum GameMode {
    Drag, Add
}

pub struct GameState {
    pub size: (f32, f32), // (width, height)

    pub origin: Point2<f32>, // Position of center on global xy-plane
    pub scale: f32, // 1 pixel corresponds to `scale` units on global xy-plane
    pub bodies: Vec<Body>,

    pub dt: f32, // Number of seconds that pass in a step
    pub paused: bool,
    pub mode: GameMode,
}

#[derive(Default)]
// Proxy through which ggez and ImGui communicate with each other
pub struct UiState {
    pub add_body: bool // Is the user currently adding a new body?
}

pub struct GameInstance {
    // Data is organised this way so `state`, `ui_state` and
    // `ui_wrapper` can be borrowed at the same time
    pub game_state: GameState,
    pub ui_state: UiState,
    pub ui_wrapper: UiWrapper
}

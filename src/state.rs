use ggez::nalgebra::Point2;

use crate::body::Body;
use crate::ui::UiWrapper;

pub struct GameState {
    pub size: (f32, f32), // (width, height)

    pub origin: Point2<f32>, // Position of center on global xy-plane
    pub scale: f32, // 1 pixel corresponds to `scale` units on global xy-plane
    pub bodies: Vec<Body>,

    pub dt: f32, // Number of seconds that pass in a step
    pub paused: bool,
    pub reversed: bool,
}

// Proxy through which ggez and ImGui communicate with each other
pub struct UiState {
    pub mouse_pos: Point2<f32>,
    pub opened: bool,
    pub scale_change: f32, // To update position field in new body dialog

    // Main menu
    pub input_scale: f32,
    pub input_dt: f32,

    // Edit body dialog
    pub selected_body_idx: Option<usize>,
    pub show_add_body: bool, // Is the user currently adding a new body?
    pub body_created: bool, // Has the new body already been created?
    pub input_mass: f32,
    pub input_pos: [f32; 2],
    pub input_v: [f32; 2],
    pub input_color: [f32; 4]
}

pub struct GameInstance {
    // Data is organised this way so `state`, `ui_state` and
    // `ui_wrapper` can be borrowed at the same time
    pub game_state: GameState,
    pub ui_state: UiState,
    pub ui_wrapper: UiWrapper
}

use ggez;
use ggez::conf;
use ggez::input::mouse;
use ggez::nalgebra::{Point2, Vector2};
use ggez::event::{self, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, Color, DrawMode, DrawParam};
use ggez::{Context, ContextBuilder, GameResult};

mod body;
use crate::body::Body;

mod ui;
use crate::ui::UiWrapper;

mod state;
use crate::state::*;

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let screen_coords = graphics::screen_coordinates(ctx);
        let game_state = GameState {
            size: (screen_coords.w, screen_coords.h),
            origin: Point2::new(0.0, 0.0),
            scale: 1e+9_f32,
            bodies: Vec::new(),
            dt: 8192.0,
            paused: false,
            mode: GameMode::Drag,
        };
        Ok(game_state)
    }

    fn local_to_global_coords(&self, pos: &Point2<f32>) -> Point2<f32> {
        let (center_x, center_y) = (self.size.0 / 2.0, self.size.1 / 2.0);
        let global_x = self.origin.x + (pos.x - center_x) * self.scale;
        let global_y = self.origin.y + (pos.y - center_y) * self.scale;
        Point2::new(global_x, global_y)
    }

    fn global_to_local_coords(&self, pos: &Point2<f32>) -> Point2<f32> {
        let (center_x, center_y) = (self.size.0 / 2.0, self.size.1 / 2.0);
        let local_x = center_x + (pos.x - self.origin.x) / self.scale;
        let local_y = center_y + (pos.y - self.origin.y) / self.scale;
        Point2::new(local_x, local_y)
    }

    fn add_body(&mut self, mass: f32, pos: Point2<f32>, v: Vector2<f32>) {
        self.bodies.push(Body {
            mass, pos, v,
            a: Vector2::new(0.0, 0.0),
            color: [1.0, 1.0, 1.0, 1.0]
        });
    }

    fn draw_body(&self, ctx: &mut Context,
                 pos: &Point2<f32>, color: &[f32; 4]) -> GameResult<()> {
        let (r, g, b, a) = (color[0], color[1], color[2], color[3]);
        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            Point2::new(0.0, 0.0),
            7.0,
            0.1,
            Color::new(r, g, b, a)
        )?;

        let dest = self.global_to_local_coords(&pos);
        graphics::draw(ctx, &circle, DrawParam::default().dest(dest))?;

        Ok(())
    }

    fn draw_bodies(&self, ctx: &mut Context) -> GameResult<()> {
        for b in &self.bodies[..] {
            self.draw_body(ctx, &b.pos, &b.color)?;
        }
        Ok(())
    }

    fn update_bodies(&mut self) {
        // Update accelerations for each body
        if self.bodies.len() >= 2 {
            for i in 0..self.bodies.len() {
                let (left, right) = self.bodies.split_at_mut(i);
                let b = &mut right[0];
                for b_ in left {
                    b.a += b.accel_towards(b_);
                    b_.a += b_.accel_towards(b);
                }
            }
        }

        // Update each body
        for b in &mut self.bodies[..] {
            b.update(self.dt);
        }
    }
}

impl UiState {
    fn new() -> UiState {
        UiState {
            mouse_pos: Point2::new(0.0, 0.0),
            opened: true,
            scale_change: 1.0,
            body_created: false,
            show_add_body: false,
            input_mass: 0.0,
            input_v: [0.0, 0.0],
            input_pos: [0.0, 0.0],
            input_color: [1.0, 1.0, 1.0, 1.0]
        }
    }
}

impl GameInstance {
    fn new(ctx: &mut Context, hidpi_factor: f32) -> GameResult<GameInstance> {
        let instance = GameInstance {
            game_state: GameState::new(ctx)?,
            ui_state: UiState::new(),
            ui_wrapper: UiWrapper::new(ctx, hidpi_factor)
        };

        Ok(instance)
    }
}

impl event::EventHandler for GameInstance {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if !self.game_state.paused {
            self.game_state.update_bodies();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        self.game_state.draw_bodies(ctx)?;
        self.ui_wrapper.update_ui(ctx, &mut self.game_state, &mut self.ui_state);
        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context,
                      key: KeyCode, mods: KeyMods, _: bool) {
        self.ui_wrapper.update_key_down(key, mods);
        match key {
            KeyCode::Q => { event::quit(ctx); return; }
            KeyCode::P => {
                if self.game_state.mode == GameMode::Drag {
                    self.game_state.paused = !self.game_state.paused;
                }
            }
            KeyCode::Left => self.game_state.dt /= 2.0,
            KeyCode::Right => self.game_state.dt *= 2.0,
            KeyCode::Up => {
                self.game_state.scale /= 2.0;
                self.ui_state.scale_change = 0.5;
            }
            KeyCode::Down => {
                self.game_state.scale *= 2.0;
                self.ui_state.scale_change = 2.0;
            }
            KeyCode::A => {
                if self.game_state.mode != GameMode::Add {
                    self.game_state.paused = true;
                }
                self.game_state.mode = GameMode::Add;
            }
            KeyCode::D => {
                if self.game_state.mode != GameMode::Drag {
                    self.game_state.paused = false;
                }
                self.game_state.mode = GameMode::Drag;
            }
            _ => ()
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context,
                    keycode: KeyCode, keymods: KeyMods) {
        self.ui_wrapper.update_key_up(keycode, keymods);
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context,
                               button: MouseButton, x: f32, y: f32) {
        self.ui_wrapper.update_mouse_down(button);
        if self.game_state.mode == GameMode::Add && !self.ui_state.show_add_body {
            self.ui_state.show_add_body = true;
            self.ui_state.mouse_pos = Point2::new(x, y);
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context,
                             button: MouseButton, _x: f32, _y: f32) {
        self.ui_wrapper.update_mouse_up(button);
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context,
                          x: f32, y: f32, dx: f32, dy: f32) {
        self.ui_wrapper.update_mouse_pos(x, y);
        if self.game_state.mode == GameMode::Drag &&
            mouse::button_pressed(ctx, mouse::MouseButton::Left) {
                self.game_state.origin += Vector2::new(-dx * self.game_state.scale,
                                                       -dy * self.game_state.scale);
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.ui_wrapper.update_scroll(x, y);
    }

    fn text_input_event(&mut self, _ctx: &mut Context, val: char) {
        self.ui_wrapper.update_text(val);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, width, height))
            .unwrap();
    }
}

fn main() -> GameResult {
    let cb = ContextBuilder::new("grav", "ggez");
    let (ctx, event_loop) = &mut cb.build()?;

    let w = 1000.0;
    let h = 800.0;
    graphics::set_mode(ctx, conf::WindowMode {
        width: w,
        height: h,
        maximized: false,
        fullscreen_type: conf::FullscreenType::Windowed,
        borderless: false,
        min_width: 0.0,
        max_width: 0.0,
        min_height: 0.0,
        max_height: 0.0,
        resizable: false
    })?;
    graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, w, h))?;

    let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor() as f32;

    let game = &mut GameInstance::new(ctx, hidpi_factor)?;
    event::run(ctx, event_loop, game)
}

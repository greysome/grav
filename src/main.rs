use std::env;
use std::path;

use ggez;
use ggez::conf;
use ggez::input::mouse;
use ggez::nalgebra::{Point2, Vector2};
use ggez::event::{self, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, Align, Text, Color, DrawMode, DrawParam, Font};
use ggez::{Context, ContextBuilder, GameResult};

mod body;
use crate::body::Body;

#[derive(PartialEq)]
enum GameMode {
    Drag, Add
}

struct Game {
    size: (f32, f32), // (width, height)
    font: Font,

    origin: Point2<f32>, // Position of center on global xy-plane
    scale: f32, // 1 pixel corresponds to `scale` units on global xy-plane
    bodies: Vec<Body>,

    dt: f32, // Number of seconds that pass in a step
    paused: bool,
    mode: GameMode
}

impl Game {
    fn new(ctx: &mut Context) -> GameResult<Game> {
        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf")?;
        let screen_coords = graphics::screen_coordinates(ctx);
        let s = Game {
            size: (screen_coords.w, screen_coords.h),
            font: font,
            origin: Point2::new(0.0, 0.0),
            scale: 1e+9_f32,
            bodies: Vec::new(),
            dt: 8192.0,
            paused: false,
            mode: GameMode::Drag
        };
        Ok(s)
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
            a: Vector2::new(0.0, 0.0)
        });
    }

    fn draw_body(&self, ctx: &mut Context, pos: &Point2<f32>) -> GameResult<()> {
        let circle = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            Point2::new(0.0, 0.0),
            7.0,
            0.1,
            graphics::WHITE
        )?;

        let dest = self.global_to_local_coords(&pos);
        graphics::draw(ctx, &circle, DrawParam::default().dest(dest))?;

        Ok(())
    }

    fn draw_bodies(&self, ctx: &mut Context) -> GameResult<()> {
        for b in &self.bodies[..] {
            self.draw_body(ctx, &b.pos)?;
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

impl event::EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if !self.paused {
            self.update_bodies();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));

        self.draw_bodies(ctx)?;

        // Paused text
        if self.paused {
            let text = Text::new(("Paused", self.font, 24.0));
            let dest = Point2::new(10.0, 10.0);
            graphics::draw(ctx, &text, DrawParam::default().dest(dest))?;
        }
        
        // Speed/scale text
        let text = format!("Speed: {}x\nScale: {}x", self.dt, self.scale);
        let mut text = Text::new((text, self.font, 24.0));
        let (w, _) = self.size;
        text.set_bounds(Point2::new(w - 10.0, 50.0), Align::Right);
        let dest = Point2::new(0.0, 10.0);
        graphics::draw(ctx, &text, DrawParam::default().dest(dest))?;

        // Mode text
        let text = match self.mode {
            GameMode::Drag => "Drag",
            GameMode::Add => "Add"
        };
        let mut text = Text::new((text, self.font, 24.0));
        let (_, h) = self.size;
        text.set_bounds(Point2::new(w, 20.0), Align::Left);
        let dest = Point2::new(10.0, h - 30.0);
        graphics::draw(ctx, &text, DrawParam::default().dest(dest))?;

        graphics::present(ctx)?;

        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context,
                      key: KeyCode, _mods: KeyMods, _: bool) {
        match key {
            KeyCode::Q => { event::quit(ctx); return; }
            KeyCode::P => self.paused = !self.paused,
            KeyCode::Left => self.dt /= 2.0,
            KeyCode::Right => self.dt *= 2.0,
            KeyCode::Up => self.scale /= 2.0,
            KeyCode::Down => self.scale *= 2.0,
            KeyCode::A => self.mode = GameMode::Add,
            KeyCode::D => self.mode = GameMode::Drag,
            _ => ()
        }
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context,
                               _button: MouseButton, x: f32, y: f32) {
        if self.mode == GameMode::Add {
            self.add_body(
                1.989e+30_f32,
                self.local_to_global_coords(&Point2::new(x, y)),
                Vector2::new(0.0, 0.0)
            );
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context,
                          _x: f32, _y: f32, dx: f32, dy: f32) {
        if self.mode == GameMode::Drag &&
            mouse::button_pressed(ctx, mouse::MouseButton::Left) {
            self.origin += Vector2::new(-dx * self.scale, -dy * self.scale);
        }
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ContextBuilder::new("grav", "ggez").add_resource_path(resource_dir);
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

    let game = &mut Game::new(ctx)?;
    event::run(ctx, event_loop, game)
}

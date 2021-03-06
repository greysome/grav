use std::time::Instant;

use ggez;
use ggez::nalgebra::{Point2, Vector2};
use ggez::event::{KeyCode, KeyMods, MouseButton};
use ggez::graphics;

use gfx_core::{handle::RenderTargetView, memory::Typed};
use gfx_device_gl;

use imgui;
use imgui::*;
use imgui_gfx_renderer::*;

use crate::state::*;

#[derive(Default)]
struct MouseState {
    pos: (i32, i32),
    /// mouse buttons: (left, right, middle)
    pressed: (bool, bool, bool),
    wheel: f32,
    wheel_h: f32,
}

pub struct UiWrapper {
    pub imgui: imgui::Context,
    pub renderer: Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>,
    hidpi_factor: f32,
    fps: f32,
    last_frame: Instant,
    mouse_state: MouseState,
}

fn reconfigure_keys(imgui: &mut imgui::Context) {
    let io = imgui.io_mut();
    io[Key::Tab] = KeyCode::Tab as _;
    io[Key::LeftArrow] = KeyCode::Left as _;
    io[Key::RightArrow] = KeyCode::Right as _;
    io[Key::UpArrow] = KeyCode::Up as _;
    io[Key::DownArrow] = KeyCode::Down as _;
    io[Key::PageUp] = KeyCode::PageUp as _;
    io[Key::PageDown] = KeyCode::PageDown as _;
    io[Key::Home] = KeyCode::Home as _;
    io[Key::End] = KeyCode::End as _;
    io[Key::Insert] = KeyCode::Insert as _;
    io[Key::Delete] = KeyCode::Delete as _;
    io[Key::Backspace] = KeyCode::Back as _;
    io[Key::Space] = KeyCode::Space as _;
    io[Key::Enter] = KeyCode::Return as _;
    io[Key::Escape] = KeyCode::Escape as _;
    io[Key::KeyPadEnter] = KeyCode::NumpadEnter as _;
    io[Key::A] = KeyCode::A as _;
    io[Key::C] = KeyCode::C as _;
    io[Key::V] = KeyCode::V as _;
    io[Key::X] = KeyCode::X as _;
    io[Key::Y] = KeyCode::Y as _;
    io[Key::Z] = KeyCode::Z as _;
}

fn build_main_menu(ui: &Ui, game_state: &mut GameState,
                   ui_state: &mut UiState, fps: f32) {
    // Some menus in main menu bar are disabled as they only serve to
    // display information
    let token = ui.push_style_color(StyleColor::TextDisabled, [1.0, 1.0, 1.0, 1.0]);
    ui.main_menu_bar(|| {
        if game_state.paused {
            ui.menu(im_str!("PAUSED"), false, || {});
        }

        if game_state.reversed {
            ui.menu(im_str!("REVERSED"), false, || {});
        }

        let scale_text = format!("Scale: {:e}x\0", game_state.scale);
        let s = unsafe {
            ImStr::from_utf8_with_nul_unchecked(scale_text.as_bytes())
        };
        ui.menu(&s, true, || {
            let input_scale = ui.input_float(im_str!(""), &mut ui_state.input_scale)
                .enter_returns_true(true);
            if input_scale.build() {
                game_state.scale = ui_state.input_scale;
            }
        });

        let dt_text = format!("Speed: {:e}x\0", game_state.dt);
        let s = unsafe {
            ImStr::from_utf8_with_nul_unchecked(dt_text.as_bytes())
        };
        ui.menu(&s, true, || {
            let input_dt = ui.input_float(im_str!(""), &mut ui_state.input_dt)
                .enter_returns_true(true);
            if input_dt.build() {
                game_state.dt = ui_state.input_dt;
            }
        });

        let fps_text = format!("FPS: {:.0}\0", fps);
        let s = unsafe {
            ImStr::from_utf8_with_nul_unchecked(fps_text.as_bytes())
        };
        ui.menu(&s, false, || {});
    });
    token.pop(&ui);
}

fn build_body_ui(ui: &Ui, game_state: &mut GameState,
                     ui_state: &mut UiState, body_idx: usize) {
    Window::new(im_str!("Body View"))
        .position([game_state.size.0 - 400.0, 20.0], Condition::Always)
        .size([400.0, game_state.size.1], Condition::Always)
        .movable(false)
        .resizable(false)
        .collapsible(false)
        .build(ui, || {
            let body = game_state.bodies[body_idx];
            ui_state.input_mass = body.mass / 1e+22_f32;
            ui_state.input_pos = [body.pos.x / game_state.scale,
                                body.pos.y / game_state.scale];
            ui_state.input_v = [body.v.x / 1000.0, body.v.y / 1000.0];
            ui_state.input_color.clone_from_slice(&body.color);

            // Update position fields accordingly when scale is changed
            if ui_state.scale_change != 1.0 {
                ui_state.input_pos = [ui_state.input_pos[0] / ui_state.scale_change,
                                    ui_state.input_pos[1] / ui_state.scale_change];
                ui_state.scale_change = 1.0;
            }

            let mass = ui.input_float(im_str!("Mass (10^22kg)"), &mut ui_state.input_mass)
                .enter_returns_true(true);
            if mass.build() {
                game_state.bodies[body_idx].mass = ui_state.input_mass * 1e+22_f32;
            }

            let pos = ui.input_float2(im_str!("Pos (to scale)"), &mut ui_state.input_pos)
                .enter_returns_true(true);
            if pos.build() {
                game_state.bodies[body_idx].pos = game_state.scale *
                    Point2::new(ui_state.input_pos[0], ui_state.input_pos[1]);
            }

            let v = ui.input_float2(im_str!("Velocity (km/s)"), &mut ui_state.input_v)
                .enter_returns_true(true);
            if v.build() {
                game_state.bodies[body_idx].v = 1000.0 *
                    Vector2::new(ui_state.input_v[0], ui_state.input_v[1]);
            }

            let cp = ColorPicker::new(im_str!("Color"), &mut ui_state.input_color)
                .inputs(false)
                .side_preview(false)
                .small_preview(false);
            if cp.build(ui) {
                game_state.bodies[body_idx].color.clone_from_slice(&ui_state.input_color);
            }

            if ui.button(im_str!("Delete"), [50.0, 20.0]) {
                game_state.bodies.remove(body_idx);
                ui_state.selected_body_idx = None;
            }

            if ui.button(im_str!("Close"), [50.0, 20.0]) {
                ui_state.selected_body_idx = None;
            }
        });
}

fn render_ui(ctx: &mut ggez::Context, ui: Ui,
             renderer: &mut Renderer<gfx_core::format::Rgba8, gfx_device_gl::Resources>) {
    let (factory, _, encoder, _, render_target) = graphics::gfx_objects(ctx);
    let draw_data = ui.render();
    renderer.render(
        &mut *factory,
        encoder,
        &mut RenderTargetView::new(render_target.clone()),
        draw_data
    ).unwrap();
}

impl UiWrapper {
    pub fn new(ctx: &mut ggez::Context, hidpi_factor: f32) -> Self {
        let mut imgui = imgui::Context::create();
        let (factory, gfx_device, _, _, _) = graphics::gfx_objects(ctx);
        let shaders = {
            let version = gfx_device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                Shaders::GlSl130
            } else {
                Shaders::GlSl110
            }
        };
        let renderer = Renderer::init(&mut imgui, &mut *factory, shaders).unwrap();

        reconfigure_keys(&mut imgui);
        Self {
            imgui,
            renderer,
            hidpi_factor,
            fps: 0.0,
            last_frame: Instant::now(),
            mouse_state: MouseState::default()
        }
    }

    pub fn update_ui(&mut self, ctx: &mut ggez::Context,
                     game_state: &mut GameState, ui_state: &mut UiState) {
        // Manually update ImGui state
        self.update_mouse();
        self.create_new_frame(ctx);

        let ui = self.imgui.frame();
        build_main_menu(&ui, game_state, ui_state, self.fps);
        if let Some(x) = ui_state.selected_body_idx {
            build_body_ui(&ui, game_state, ui_state, x);
        }

        render_ui(ctx, ui, &mut self.renderer);
    }

    fn create_new_frame(&mut self, ctx: &mut ggez::Context) {
        let io = self.imgui.io_mut();
        let now = Instant::now();
        let delta = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.fps = 1.0 / delta_s;
        self.last_frame = now;

        let (draw_width, draw_height) = graphics::drawable_size(ctx);
        io.display_size = [draw_width, draw_height];
        io.display_framebuffer_scale =
            [self.hidpi_factor, self.hidpi_factor];
        io.delta_time = delta_s;
    }

    //
    // The functions below manually populate ImGui mouse/key state by
    // checking ggez events
    //
    fn update_mouse(&mut self) {
        let io = self.imgui.io_mut();

        io.mouse_pos = [self.mouse_state.pos.0 as f32, self.mouse_state.pos.1 as f32];
        io.mouse_down = [
            self.mouse_state.pressed.0,
            self.mouse_state.pressed.1,
            self.mouse_state.pressed.2,
            false,
            false,
        ];

        io.mouse_wheel = self.mouse_state.wheel;
        self.mouse_state.wheel = 0.0;

        io.mouse_wheel_h = self.mouse_state.wheel_h;
        self.mouse_state.wheel_h = 0.0;
    }

    pub fn update_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_state.pos = (x as i32, y as i32);
    }

    pub fn update_mouse_down(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.mouse_state.pressed.0 = true,
            MouseButton::Right => self.mouse_state.pressed.1 = true,
            MouseButton::Middle => self.mouse_state.pressed.2 = true,
            _ => ()
        }
    }

    pub fn update_mouse_up(&mut self, button: MouseButton) {
        match button {
            MouseButton::Left => self.mouse_state.pressed.0 = false,
            MouseButton::Right => self.mouse_state.pressed.1 = false,
            MouseButton::Middle => self.mouse_state.pressed.2 = false,
            _ => ()
        }
    }

    pub fn update_key_down(&mut self, key: KeyCode, mods: KeyMods) {
        let io = self.imgui.io_mut();
        io.key_shift = mods.contains(KeyMods::SHIFT);
        io.key_ctrl = mods.contains(KeyMods::CTRL);
        io.key_alt = mods.contains(KeyMods::ALT);
        io.keys_down[key as usize] = true;
    }

    pub fn update_key_up(&mut self, key: KeyCode, mods: KeyMods) {
        let io = self.imgui.io_mut();
        if mods.contains(KeyMods::SHIFT) {
            io.key_shift = false;
        }
        if mods.contains(KeyMods::CTRL) {
            io.key_ctrl = false;
        }
        if mods.contains(KeyMods::ALT) {
            io.key_alt = false;
        }
        io.keys_down[key as usize] = false;
    }

    pub fn update_text(&mut self, val: char) {
        self.imgui.io_mut().add_input_character(val);
    }

    pub fn update_scroll(&mut self, x: f32, y: f32) {
        self.mouse_state.wheel += y;
        self.mouse_state.wheel_h += x;
    }
}

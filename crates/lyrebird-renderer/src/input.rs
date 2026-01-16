use std::collections::HashSet;

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

/// A manager for input.
#[derive(Default, Clone)] 
pub struct InputManager {
    /// Any events that may have not being covered, you can cover yourself. 
    pub latest_event: Option<WindowEvent>,

    /// Keys currently held down (tracked via `KeyCode`).
    pub keys_down: HashSet<KeyCode>,
    /// Mouse buttons currently held down.
    pub mouse_buttons_down: HashSet<MouseButton>,
    /// Most recent cursor position.
    pub cursor_position: Option<PhysicalPosition<f64>>,
    /// Scroll delta accumulated since last `reset_frame_deltas()`.
    pub scroll_delta: (f32, f32),
    /// Last key event this frame (if any).
    pub last_key: Option<(KeyCode, ElementState)>,
    /// Last mouse button event this frame (if any).
    pub last_mouse_button: Option<(MouseButton, ElementState)>,
}

impl InputManager {
    /// Call once per frame if you want `scroll_delta`, `last_key`, and
    /// `last_mouse_button` to represent only that frame.
    pub fn reset_frame_deltas(&mut self) {
        self.scroll_delta = (0.0, 0.0);
        self.last_key = None;
        self.last_mouse_button = None;
    }

    /// Returns true if this `WindowEvent` is one we treat as user input.
    pub fn is_input_event(event: &WindowEvent) -> bool {
        matches!(
            event,
            WindowEvent::KeyboardInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::ModifiersChanged(_)
        )
    }

    pub(crate) fn poll(&mut self, event: WindowEvent) {
        match &event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    self.last_key = Some((code, event.state));
                    match event.state {
                        ElementState::Pressed => {
                            self.keys_down.insert(code);
                        }
                        ElementState::Released => {
                            self.keys_down.remove(&code);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(*position);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.last_mouse_button = Some((*button, *state));
                match state {
                    ElementState::Pressed => {
                        self.mouse_buttons_down.insert(*button);
                    }
                    ElementState::Released => {
                        self.mouse_buttons_down.remove(button);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    self.scroll_delta.0 += *x;
                    self.scroll_delta.1 += *y;
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    self.scroll_delta.0 += pos.x as f32;
                    self.scroll_delta.1 += pos.y as f32;
                }
            },
            _ => {}
        }
        self.latest_event = Some(event);
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&button)
    }

    pub fn take_latest_event(&mut self) -> Option<WindowEvent> {
        self.latest_event.take()
    }
}
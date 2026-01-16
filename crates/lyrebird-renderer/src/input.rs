use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use parking_lot::Mutex;

#[cfg(not(target_arch = "wasm32"))]
use gilrs::{Axis, Button, EventType, GamepadId, Gilrs};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug, Clone)]
pub struct GamepadInfo {
    pub name: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Default)]
pub struct GamepadState {
    pub info: GamepadInfo,
    pub buttons_down: HashSet<Button>,
    pub button_values: HashMap<Button, f32>,
    pub axes: HashMap<Axis, f32>,
}

#[derive(Debug, Clone)]
pub struct GamepadsSnapshot {
    pub gamepads: HashMap<GamepadId, GamepadState>,
}

impl Default for GamepadInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_connected: false,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_axis_value(value: f32) -> f32 {
    // gilrs can occasionally produce NaN on device quirks; keep consumers safe.
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

#[derive(Default)]
struct GamepadFrameDeltas {
    just_pressed: HashSet<(GamepadId, Button)>,
    just_released: HashSet<(GamepadId, Button)>,
}

struct InputInner {
    #[cfg(not(target_arch = "wasm32"))]
    gilrs: Gilrs,

    /// Any events that may have not being covered, you can cover yourself.
    latest_event: Option<WindowEvent>,

    /// Keys currently held down (tracked via `KeyCode`).
    keys_down: HashSet<KeyCode>,
    /// Mouse buttons currently held down.
    mouse_buttons_down: HashSet<MouseButton>,
    /// Most recent cursor position.
    cursor_position: Option<PhysicalPosition<f64>>,
    /// Scroll delta accumulated since last `reset_frame_deltas()`.
    scroll_delta: (f32, f32),
    /// Last key event this frame (if any).
    last_key: Option<(KeyCode, ElementState)>,
    /// Last mouse button event this frame (if any).
    last_mouse_button: Option<(MouseButton, ElementState)>,

    #[cfg(not(target_arch = "wasm32"))]
    gamepads: HashMap<GamepadId, GamepadState>,
    #[cfg(not(target_arch = "wasm32"))]
    gamepad_frame: GamepadFrameDeltas,
}

impl InputInner {
    fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let gilrs = Gilrs::new().expect("failed to initialize gilrs");
            let mut gamepads: HashMap<GamepadId, GamepadState> = HashMap::new();

            // Seed state with already-connected controllers (controllers present before launch).
            for (id, gamepad) in gilrs.gamepads() {
                let info = GamepadInfo {
                    name: gamepad.name().to_string(),
                    is_connected: gamepad.is_connected(),
                };

                let state = GamepadState {
                    info,
                    ..Default::default()
                };

                gamepads.insert(id, state);
            }

            Self {
                gilrs,
                latest_event: None,
                keys_down: HashSet::new(),
                mouse_buttons_down: HashSet::new(),
                cursor_position: None,
                scroll_delta: (0.0, 0.0),
                last_key: None,
                last_mouse_button: None,
                gamepads,
                gamepad_frame: GamepadFrameDeltas::default(),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self {
                latest_event: None,
                keys_down: HashSet::new(),
                mouse_buttons_down: HashSet::new(),
                cursor_position: None,
                scroll_delta: (0.0, 0.0),
                last_key: None,
                last_mouse_button: None,
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_gamepad_info(&mut self, id: GamepadId) {
        let gamepad = self.gilrs.gamepad(id);
        let entry = self.gamepads.entry(id).or_insert_with(|| GamepadState {
            info: GamepadInfo::default(),
            ..Default::default()
        });
        entry.info.name = gamepad.name().to_string();
        entry.info.is_connected = gamepad.is_connected();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn pump_gilrs_events(&mut self) {
        self.gamepad_frame.just_pressed.clear();
        self.gamepad_frame.just_released.clear();

        while let Some(ev) = self.gilrs.next_event() {
            let id = ev.id;
            match ev.event {
                EventType::Connected => {
                    self.refresh_gamepad_info(id);
                }
                EventType::Disconnected => {
                    self.refresh_gamepad_info(id);
                }
                EventType::ButtonPressed(button, _) => {
                    self.refresh_gamepad_info(id);
                    let state = self.gamepads.entry(id).or_default();
                    state.buttons_down.insert(button);
                    self.gamepad_frame.just_pressed.insert((id, button));
                }
                EventType::ButtonReleased(button, _) => {
                    self.refresh_gamepad_info(id);
                    if let Some(state) = self.gamepads.get_mut(&id) {
                        state.buttons_down.remove(&button);
                    }
                    self.gamepad_frame.just_released.insert((id, button));
                }
                EventType::ButtonChanged(button, value, _) => {
                    self.refresh_gamepad_info(id);
                    let state = self.gamepads.entry(id).or_default();
                    state.button_values.insert(button, value.clamp(0.0, 1.0));
                }
                EventType::AxisChanged(axis, value, _) => {
                    self.refresh_gamepad_info(id);
                    let state = self.gamepads.entry(id).or_default();
                    state.axes.insert(axis, normalize_axis_value(value));
                }
                _ => {}
            }
        }
    }
}

/// A manager for input.
pub struct InputManager {
    inner: Arc<Mutex<InputInner>>,
}

impl Clone for InputManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InputInner::new())),
        }
    }
}

impl InputManager {
    /// Call once per frame if you want `scroll_delta`, `last_key`, and
    /// `last_mouse_button` to represent only that frame.
    pub fn reset_frame_deltas(&self) {
        let mut inner = self.inner.lock();
        inner.scroll_delta = (0.0, 0.0);
        inner.last_key = None;
        inner.last_mouse_button = None;
    }

    /// Poll gamepad events (gilrs). Call once per frame.
    ///
    /// This is separate from `poll_window_event` because gamepads are not driven
    /// by winit window events.
    pub fn update_gamepads(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.lock().pump_gilrs_events();
        }
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

    pub(crate) fn poll(&self, event: WindowEvent) {
        let mut inner = self.inner.lock();
        match &event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    inner.last_key = Some((code, event.state));
                    match event.state {
                        ElementState::Pressed => {
                            inner.keys_down.insert(code);
                        }
                        ElementState::Released => {
                            inner.keys_down.remove(&code);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                inner.cursor_position = Some(*position);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                inner.last_mouse_button = Some((*button, *state));
                match state {
                    ElementState::Pressed => {
                        inner.mouse_buttons_down.insert(*button);
                    }
                    ElementState::Released => {
                        inner.mouse_buttons_down.remove(button);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    inner.scroll_delta.0 += *x;
                    inner.scroll_delta.1 += *y;
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    inner.scroll_delta.0 += pos.x as f32;
                    inner.scroll_delta.1 += pos.y as f32;
                }
            },
            _ => {}
        }
        inner.latest_event = Some(event);
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.inner.lock().keys_down.contains(&key)
    }

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.inner.lock().mouse_buttons_down.contains(&button)
    }

    pub fn cursor_position(&self) -> Option<PhysicalPosition<f64>> {
        self.inner.lock().cursor_position
    }

    pub fn scroll_delta(&self) -> (f32, f32) {
        self.inner.lock().scroll_delta
    }

    pub fn last_key(&self) -> Option<(KeyCode, ElementState)> {
        self.inner.lock().last_key
    }

    pub fn last_mouse_button(&self) -> Option<(MouseButton, ElementState)> {
        self.inner.lock().last_mouse_button
    }

    pub fn take_latest_event(&self) -> Option<WindowEvent> {
        self.inner.lock().latest_event.take()
    }

    // --------------------
    // Gamepad query helpers
    // --------------------

    #[cfg(not(target_arch = "wasm32"))]
    pub fn gamepads_snapshot(&self) -> GamepadsSnapshot {
        let inner = self.inner.lock();
        GamepadsSnapshot {
            gamepads: inner.gamepads.clone(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn gamepads_snapshot(&self) -> GamepadsSnapshot {
        GamepadsSnapshot {
            gamepads: HashMap::new(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_button_pressed(&self, id: GamepadId, button: Button) -> bool {
        self.inner
            .lock()
            .gamepads
            .get(&id)
            .is_some_and(|g| g.buttons_down.contains(&button))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn is_button_pressed(&self, _id: GamepadId, _button: Button) -> bool {
        false
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn button_value(&self, id: GamepadId, button: Button) -> f32 {
        self.inner
            .lock()
            .gamepads
            .get(&id)
            .and_then(|g| g.button_values.get(&button).copied())
            .unwrap_or(0.0)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn button_value(&self, _id: GamepadId, _button: Button) -> f32 {
        0.0
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn axis_value(&self, id: GamepadId, axis: Axis) -> f32 {
        self.inner
            .lock()
            .gamepads
            .get(&id)
            .and_then(|g| g.axes.get(&axis).copied())
            .unwrap_or(0.0)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn axis_value(&self, _id: GamepadId, _axis: Axis) -> f32 {
        0.0
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn was_button_just_pressed(&self, id: GamepadId, button: Button) -> bool {
        self.inner
            .lock()
            .gamepad_frame
            .just_pressed
            .contains(&(id, button))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn was_button_just_pressed(&self, _id: GamepadId, _button: Button) -> bool {
        false
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn was_button_just_released(&self, id: GamepadId, button: Button) -> bool {
        self.inner
            .lock()
            .gamepad_frame
            .just_released
            .contains(&(id, button))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn was_button_just_released(&self, _id: GamepadId, _button: Button) -> bool {
        false
    }
}
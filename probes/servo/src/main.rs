/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Based on Servo v0.7.0's components/servo/examples/winit_minimal.rs.
// Input forwarding follows ports/servoshell/desktop/headed_window.rs.
mod input;

use std::cell::Cell;
use std::error::Error;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

use euclid::Scale;
use servo::{
    CompositionEvent, CompositionState, Cursor, DevicePoint, EmbedderControl, EmbedderControlId,
    EventLoopWaker, ImeEvent, InputEvent, MouseButtonAction, MouseButtonEvent,
    MouseLeftViewportEvent, MouseMoveEvent, RenderingContext, Servo, ServoBuilder, WebView,
    WebViewBuilder, WebViewDelegate, WheelDelta, WheelEvent, WheelMode, WindowRenderingContext,
};
use url::Url;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Ime, MouseScrollDelta, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::ModifiersState;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::{Window, WindowId};

fn main() -> Result<(), Box<dyn Error>> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App {
        waker: Waker(event_loop.create_proxy()),
        state: None,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}

struct App {
    waker: Waker,
    state: Option<State>,
}

struct State {
    servo: Servo,
    webview: WebView,
    rendering_context: Rc<WindowRenderingContext>,
    delegate: Rc<Delegate>,
    mouse_position: DevicePoint,
    modifiers: ModifiersState,
    composing: bool,
}

// The delegate owns only the window, so there is no WebView/delegate ownership cycle.
struct Delegate {
    window: Rc<Window>,
    input_method: Cell<Option<EmbedderControlId>>,
    animating: Cell<bool>,
}

impl WebViewDelegate for Delegate {
    fn notify_new_frame_ready(&self, _: WebView) {
        self.window.request_redraw();
    }

    fn notify_animating_changed(&self, _: WebView, animating: bool) {
        self.animating.set(animating);
    }

    fn notify_cursor_changed(&self, _: WebView, cursor: Cursor) {
        input::set_cursor(&self.window, cursor);
    }

    fn show_embedder_control(&self, _: WebView, control: EmbedderControl) {
        if let EmbedderControl::InputMethod(input_method) = control {
            self.input_method.set(Some(input_method.id()));
            let rect = input_method.position();
            self.window.set_ime_allowed(true);
            self.window.set_ime_cursor_area(
                PhysicalPosition::new(rect.min.x, rect.min.y),
                PhysicalSize::new(rect.width(), rect.height()),
            );
        }
    }

    fn hide_embedder_control(&self, _: WebView, id: EmbedderControlId) {
        if self.input_method.get() == Some(id) {
            self.input_method.set(None);
            self.window.set_ime_allowed(false);
        }
    }
}

impl ApplicationHandler<Wake> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let window = Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("QuantumView - Servo")
                        .with_inner_size(LogicalSize::new(1000.0, 720.0)),
                )
                .expect("Failed to create window"),
        );
        let rendering_context = Rc::new(
            WindowRenderingContext::new(
                event_loop.display_handle().expect("No display handle"),
                window.window_handle().expect("No window handle"),
                window.inner_size(),
            )
            .expect("Failed to create rendering context"),
        );
        rendering_context
            .make_current()
            .expect("Failed to make rendering context current");
        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(self.waker.clone()))
            .build();
        servo.setup_logging();
        let delegate = Rc::new(Delegate {
            window,
            input_method: Cell::new(None),
            animating: Cell::new(false),
        });
        let ui = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../ui/index.html")
            .canonicalize()
            .expect("Shared ui/index.html is missing");
        let webview = WebViewBuilder::new(&servo, rendering_context.clone())
            .url(Url::from_file_path(ui).expect("Invalid UI file path"))
            .hidpi_scale_factor(Scale::new(delegate.window.scale_factor() as f32))
            .delegate(delegate.clone())
            .build();
        webview.set_focused(delegate.window.has_focus());
        self.state = Some(State {
            servo,
            webview,
            rendering_context,
            delegate,
            mouse_position: DevicePoint::zero(),
            modifiers: ModifiersState::empty(),
            composing: false,
        });
        self.state.as_ref().unwrap().servo.spin_event_loop();
    }

    fn user_event(&mut self, _: &ActiveEventLoop, _: Wake) {
        if let Some(state) = &self.state {
            state.servo.spin_event_loop();
        }
    }

    fn new_events(&mut self, _: &ActiveEventLoop, cause: StartCause) {
        if let Some(state) = &self.state
            && matches!(cause, StartCause::ResumeTimeReached { .. })
            && state.delegate.animating.get()
        {
            state.servo.spin_event_loop();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let animating = self
            .state
            .as_ref()
            .is_some_and(|state| state.delegate.animating.get());
        event_loop.set_control_flow(if animating {
            ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16))
        } else {
            ControlFlow::Wait
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(state) = &mut self.state else {
            return;
        };
        let webview = &state.webview;
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                webview.paint();
                state.rendering_context.present();
            }
            WindowEvent::Resized(size) => webview.resize(size),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                webview.set_hidpi_scale_factor(Scale::new(scale_factor as f32));
                webview.resize(state.delegate.window.inner_size());
            }
            WindowEvent::Focused(focused) => {
                webview.set_focused(focused);
                if !focused {
                    state.modifiers = ModifiersState::empty();
                    state.composing = false;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                state.mouse_position = DevicePoint::new(position.x as f32, position.y as f32);
                webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
                    state.mouse_position.into(),
                )));
            }
            WindowEvent::CursorLeft { .. } => {
                webview.notify_input_event(InputEvent::MouseLeftViewport(
                    MouseLeftViewportEvent::default(),
                ));
            }
            WindowEvent::MouseInput {
                state: key, button, ..
            } => {
                let action = match key {
                    ElementState::Pressed => MouseButtonAction::Down,
                    ElementState::Released => MouseButtonAction::Up,
                };
                webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                    action,
                    input::mouse_button(button),
                    state.mouse_position.into(),
                )));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Keep discrete line input separate from a trackpad's pixel input.
                let (x, y, mode) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        (f64::from(x), f64::from(y), WheelMode::DeltaLine)
                    }
                    MouseScrollDelta::PixelDelta(delta) => {
                        (delta.x, delta.y, WheelMode::DeltaPixel)
                    }
                };
                if std::env::var_os("QUANTUMVIEW_WHEEL_TRACE").is_some_and(|value| value == "1") {
                    let mode_name = match &mode {
                        WheelMode::DeltaLine => "line",
                        WheelMode::DeltaPixel => "pixel",
                        _ => "other",
                    };
                    eprintln!("[wheel-trace] host wheel: mapped=({x}, {y}, {mode_name})");
                }
                webview.notify_input_event(InputEvent::Wheel(WheelEvent::new(
                    WheelDelta { x, y, z: 0.0, mode },
                    state.mouse_position.into(),
                )));
            }
            WindowEvent::ModifiersChanged(modifiers) => state.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput { event, .. } => {
                webview.notify_input_event(InputEvent::Keyboard(input::keyboard_event(
                    &event,
                    state.modifiers,
                    state.composing,
                )));
            }
            WindowEvent::Ime(ime) => {
                let composition = match ime {
                    Ime::Enabled => {
                        state.composing = true;
                        Some((CompositionState::Start, String::new()))
                    }
                    Ime::Preedit(text, _) => {
                        state.composing = true;
                        Some((CompositionState::Update, text))
                    }
                    Ime::Commit(text) => {
                        state.composing = false;
                        Some((CompositionState::End, text))
                    }
                    Ime::Disabled => {
                        state.composing = false;
                        // On Windows this also follows a normal composition commit.
                        // It must not dismiss the editor or clear its focus.
                        None
                    }
                };
                if let Some((state, data)) = composition {
                    webview.notify_input_event(InputEvent::Ime(ImeEvent::Composition(
                        CompositionEvent { state, data },
                    )));
                }
            }
            _ => {}
        }
        state.servo.spin_event_loop();
    }
}

#[derive(Clone)]
struct Waker(EventLoopProxy<Wake>);

#[derive(Debug)]
struct Wake;

impl EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }

    fn wake(&self) {
        let _ = self.0.send_event(Wake);
    }
}

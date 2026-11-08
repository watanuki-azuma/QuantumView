#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_os = "windows"))]
compile_error!("The WebView2 baseline is Windows-only.");

use std::error::Error;
use std::path::Path;
use url::Url;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    // Drop the WebView before its parent window.
    webview: Option<wry::WebView>,
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("QuantumView - WebView2")
                    .with_inner_size(LogicalSize::new(1000.0, 720.0)),
            )
            .expect("Could not create window");
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../ui/index.html")
            .canonicalize()
            .expect("Could not find ui/index.html");
        let url = Url::from_file_path(path).expect("UI path must be absolute");
        let webview = wry::WebViewBuilder::new()
            .with_url(url.as_str())
            .build(&window)
            .expect("Could not create WebView2; install the WebView2 Runtime");
        self.webview = Some(webview);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            self.webview = None;
            self.window = None;
            event_loop.exit();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App::default())?;
    Ok(())
}

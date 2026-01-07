use std::sync::Mutex;

pub use crate::capturer::engine::linux::portal::StreamVardict;
use crate::{capturer::engine::linux::portal::ScreenCastPortal, targets::Window, Display, Target};

/// Prepare a stream for capture and use the stream_id.
/// This struct is Send + Sync safe.
pub struct CaptureStreamBuilder {
    /// Hold connection to the portal to keep the session alive.
    /// Wrapped in Mutex to make the struct Sync.
    _connection: Mutex<dbus::blocking::Connection>,
    pub stream_id: u32,
    pub stream_var_dict: StreamVardict,
}

// Safety: The connection is only held to keep the portal session alive.
// It's wrapped in a Mutex so concurrent access is safe.
unsafe impl Sync for CaptureStreamBuilder {}

impl CaptureStreamBuilder {
    pub fn new() -> Self {
        let connection =
            dbus::blocking::Connection::new_session().expect("Failed to create dbus connection");

        let portal = ScreenCastPortal::new(&connection)
            .show_cursor(true)
            .expect("Unsupported cursor mode");

        let stream = portal
            .create_stream()
            .expect("Failed to create screencast stream");

        let stream_id = stream.pw_node_id();

        Self {
            _connection: Mutex::new(connection),
            stream_id,
            stream_var_dict: stream.stream_dict(),
        }
    }

    pub fn get_target(&self) -> Target {
        let stream_var_dict = self.stream_var_dict.clone();
        let is_display = stream_var_dict.is_display.unwrap_or(true);
        let id = self.stream_id;
        let size = stream_var_dict.size;
        let position = stream_var_dict.position;
        // Cant get the title of selected target, so we use a fallback
        let title = format!(
            "{}:{}:{}",
            match stream_var_dict.is_display {
                Some(true) => "Display",
                Some(false) => "Window",
                None => "unknown",
            },
            id.clone(),
            stream_var_dict
                .size
                .map(|(w, h)| format!("{}x{}", w, h))
                .unwrap_or("WxH".to_string()),
        );
        if is_display {
            return Target::Display(Display {
                id: id,
                title,
                size: size.map(|(w, h)| (w as u32, h as u32)),
                position: position.map(|(x, y)| (x as u32, y as u32)),
            });
        } else {
            return Target::Window(Window {
                id,
                title,
                size: size.map(|(w, h)| (w as u32, h as u32)),
                position: position.map(|(x, y)| (x as u32, y as u32)),
            });
        }
    }
}

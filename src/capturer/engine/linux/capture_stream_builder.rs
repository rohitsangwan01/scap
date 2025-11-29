use std::sync::Mutex;

use crate::capturer::engine::linux::portal::ScreenCastPortal;
pub use crate::capturer::engine::linux::portal::StreamVardict;

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
}

use crate::capturer::engine::linux::portal::ScreenCastPortal;
pub use crate::capturer::engine::linux::portal::StreamVardict;

/// Prepare a stream for capture and use the stream_id
pub struct CaptureStreamBuilder {
    /// Hold connection to the portal
    _connection: dbus::blocking::Connection,
    pub stream_id: u32,
    pub stream_var_dict: StreamVardict,
}

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
            _connection: connection,
            stream_id,
            stream_var_dict: stream.stream_dict(),
        }
    }
}

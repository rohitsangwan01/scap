use core_graphics_helmer_fork::display::{CGDisplay, CGDisplayMode};

pub trait DirectDisplayIdExt {
    fn display_mode(&self) -> Option<CGDisplayMode>;
}

impl DirectDisplayIdExt for core_graphics_helmer_fork::display::CGDirectDisplayID {
    #[inline]
    fn display_mode(&self) -> Option<CGDisplayMode> {
        CGDisplay::new(*self).display_mode()
    }
}

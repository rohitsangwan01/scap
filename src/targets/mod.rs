#[cfg(target_os = "macos")]
mod mac;

#[cfg(target_os = "macos")]
pub(crate) use mac::get_display_name;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "linux")]
mod linux;

#[derive(Debug, Clone)]
pub struct Window {
    pub id: u32,
    pub title: String,
    pub size: Option<(u32, u32)>,
    pub position: Option<(u32, u32)>,

    #[cfg(target_os = "windows")]
    pub raw_handle: windows::Win32::Foundation::HWND,

    #[cfg(target_os = "macos")]
    pub raw_handle: cidre::cg::WindowId,
}

#[derive(Debug, Clone)]
pub struct Display {
    pub id: u32,
    pub title: String,
    pub size: Option<(u32, u32)>,
    pub position: Option<(u32, u32)>,

    #[cfg(target_os = "windows")]
    pub raw_handle: windows::Win32::Graphics::Gdi::HMONITOR,

    #[cfg(target_os = "macos")]
    pub raw_handle: cidre::cg::DirectDisplayId,
}

#[derive(Debug, Clone)]
pub enum Target {
    Window(Window),
    Display(Display),
}

impl Target {
    pub fn id(&self) -> u32 {
        match self {
            Target::Window(window) => window.id,
            Target::Display(display) => display.id,
        }
    }

    pub fn title(&self) -> String {
        match self {
            Target::Window(window) => window.title.clone(),
            Target::Display(display) => display.title.clone(),
        }
    }

    pub fn size(&self) -> Option<(u32, u32)> {
        match self {
            Target::Window(window) => window.size,
            Target::Display(display) => display.size,
        }
    }

    pub fn position(&self) -> Option<(u32, u32)> {
        match self {
            Target::Window(window) => window.position,
            Target::Display(display) => display.position,
        }
    }
}

/// Returns a list of targets that can be captured
pub fn get_all_targets() -> Vec<Target> {
    #[cfg(target_os = "macos")]
    return mac::get_all_targets();

    #[cfg(target_os = "windows")]
    return win::get_all_targets();

    #[cfg(target_os = "linux")]
    return linux::get_all_targets();
}

pub fn get_scale_factor(target: &Target) -> f64 {
    #[cfg(target_os = "macos")]
    return mac::get_scale_factor(target);

    #[cfg(target_os = "windows")]
    return win::get_scale_factor(target);

    #[cfg(target_os = "linux")]
    return 1.0;
}

pub fn get_main_display() -> Display {
    #[cfg(target_os = "macos")]
    return mac::get_main_display();

    #[cfg(target_os = "windows")]
    return win::get_main_display();

    #[cfg(target_os = "linux")]
    unreachable!();
}

pub fn get_target_dimensions(target: &Target) -> (u64, u64) {
    #[cfg(target_os = "macos")]
    return mac::get_target_dimensions(target);

    #[cfg(target_os = "windows")]
    return win::get_target_dimensions(target);

    #[cfg(target_os = "linux")]
    unreachable!();
}

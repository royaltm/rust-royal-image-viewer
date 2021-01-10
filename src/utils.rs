#![allow(unused_imports)]
use std::{borrow::Cow, error::Error, fmt, ptr};

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct ExitError {
    message: &'static str,
    exit_code: i32
}

impl ExitError {
    pub fn new(message: &'static str, exit_code: i32) -> Self {
        ExitError { message, exit_code }
    }
}

impl fmt::Display for ExitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl Error for ExitError {}

impl ExitError {
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
}

pub fn err_code(msg: &'static str, code: i32) -> Result<()> {
    Err(ExitError::new(msg, code).into())
}


#[cfg(not(windows))]
pub fn set_dpi_awareness() -> core::result::Result<(), String> { Ok(()) }

#[cfg(windows)]
pub fn set_dpi_awareness() -> core::result::Result<(), String> {
    use winapi::{shared::winerror::{E_INVALIDARG, S_OK},
                 um::shellscalingapi::{GetProcessDpiAwareness, SetProcessDpiAwareness, PROCESS_DPI_UNAWARE,
                                       PROCESS_PER_MONITOR_DPI_AWARE}};

    match unsafe { SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE) } {
        S_OK => Ok(()),
        E_INVALIDARG => Err("Could not set DPI awareness.".into()),
        _ => {
            let mut awareness = PROCESS_DPI_UNAWARE;
            match unsafe { GetProcessDpiAwareness(ptr::null_mut(), &mut awareness) } {
                S_OK if awareness == PROCESS_PER_MONITOR_DPI_AWARE => Ok(()),
                _ => Err("Please disable DPI awareness override in program properties.".into()),
            }
        },
    }
}

#[cfg(not(windows))]
pub fn free_console_window() {}

#[cfg(windows)]
pub fn free_console_window() {
    unsafe { winapi::um::wincon::FreeConsole() };
}

// #[cfg(not(windows))]
// pub fn attach_console_window() {}

// #[cfg(windows)]
// pub fn attach_console_window() {
//     use winapi::um::wincon;
//     unsafe { wincon::AttachConsole(wincon::ATTACH_PARENT_PROCESS) };
// }

// #[cfg(not(windows))]
// pub fn hide_console_window() {}

// #[cfg(windows)]
// pub fn hide_console_window() {
//     use winapi::um::{wincon::GetConsoleWindow, winuser::{ShowWindow, SW_HIDE}};

//     let window = unsafe { GetConsoleWindow() };
//     // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
//     if window != std::ptr::null_mut() {
//         unsafe {
//             ShowWindow(window, SW_HIDE);
//         }
//     }
// }
#![allow(unused_imports)]
use std::{borrow::Cow, error::Error, ptr};

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

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
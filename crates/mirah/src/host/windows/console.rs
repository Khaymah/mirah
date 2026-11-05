#![allow(
    unsafe_code,
    reason = "GetConsoleMode and SetConsoleMode have no safe wrapper"
)]

use std::sync::Mutex;
use windows_sys::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Console::{
    CONSOLE_MODE, ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
    GetConsoleMode, GetStdHandle, STD_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, SetConsoleMode,
};

static SAVED_MODES: Mutex<Vec<(STD_HANDLE, CONSOLE_MODE)>> = Mutex::new(Vec::new());

fn console(which: STD_HANDLE) -> Option<HANDLE> {
    let handle = unsafe { GetStdHandle(which) };
    (!handle.is_null() && handle != INVALID_HANDLE_VALUE).then_some(handle)
}

fn mode(handle: HANDLE) -> Option<CONSOLE_MODE> {
    let mut mode: CONSOLE_MODE = 0;
    (unsafe { GetConsoleMode(handle, &raw mut mode) } != 0).then_some(mode)
}

fn set_mode(handle: HANDLE, mode: CONSOLE_MODE) -> bool {
    unsafe { SetConsoleMode(handle, mode) != 0 }
}

pub(crate) fn enable_virtual_terminal() {
    let Ok(mut saved) = SAVED_MODES.lock() else {
        return;
    };
    for (which, flag) in [
        (STD_INPUT_HANDLE, ENABLE_VIRTUAL_TERMINAL_INPUT),
        (STD_OUTPUT_HANDLE, ENABLE_VIRTUAL_TERMINAL_PROCESSING),
    ] {
        let Some(handle) = console(which) else {
            continue;
        };
        let Some(original) = mode(handle) else {
            tracing::debug!(which, "standard handle is not a console");
            continue;
        };
        if set_mode(handle, original | flag) {
            saved.push((which, original));
        } else {
            tracing::warn!(which, "could not enable virtual terminal mode");
        }
    }
}

pub(crate) fn restore_console_modes() {
    let Ok(mut saved) = SAVED_MODES.lock() else {
        return;
    };
    for (which, original) in saved.drain(..) {
        if let Some(handle) = console(which) {
            set_mode(handle, original);
        }
    }
}

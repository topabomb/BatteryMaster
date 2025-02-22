use tauri::{AppHandle, Manager};

#[cfg(windows)]
pub fn is_admin() -> bool {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::GetCurrentProcess;
    use winapi::um::processthreadsapi::OpenProcessToken;
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION};

    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), 0x0008, &mut token) == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
        let status = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size,
            &mut size,
        );

        CloseHandle(token); // 必须关闭句柄
        status != 0 && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(windows))]
pub fn is_admin() -> bool {
    false // 非Windows系统返回false
}
#[cfg(windows)]
pub fn elevate_self() {
    use std::iter;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::shellapi::ShellExecuteW;
    use winapi::um::winuser::SW_SHOW;

    let exe_path = std::env::current_exe().expect("获取程序路径失败");
    let os_str = exe_path
        .as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();

    let verb: Vec<u16> = "runas\0".encode_utf16().collect();
    let params = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .join(" ")
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();

    unsafe {
        let result = ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            os_str.as_ptr(),
            params.as_ptr(),
            std::ptr::null(),
            SW_SHOW,
        );

        if result as i32 > 32 {
            std::process::exit(0);
        }
    }
}
#[cfg(not(windows))]
pub fn elevate_self() {}

pub fn active_window(app: &AppHandle, name: &str) {
    if let Some(window) = app.get_webview_window(name) {
        if window.is_minimized().unwrap() {
            window.unminimize().unwrap();
        }
        if window.is_maximized().unwrap() {
            window.unmaximize().unwrap();
        }
        window.show().unwrap();
        window.set_focus().unwrap();
    }
}

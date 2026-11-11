use std::ffi::c_void;
use std::ptr;
use std::thread::sleep;
use std::time::Duration;

pub type HWND = *mut c_void;
pub type HANDLE = *mut c_void;
pub type HGLOBAL = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HICON = *mut c_void;
pub type HCURSOR = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type ATOM = u16;
pub type BOOL = i32;
pub type UINT = u32;
pub type DWORD = u32;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;

pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;

pub const CF_UNICODETEXT: UINT = 13;
pub const GMEM_MOVEABLE: UINT = 0x0002;
pub const ERROR_ALREADY_EXISTS: DWORD = 183;

pub const WM_DESTROY: UINT = 0x0002;
pub const WM_CLIPBOARDUPDATE: UINT = 0x031D;

pub const HWND_MESSAGE: HWND = (-3isize) as HWND;

#[repr(C)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub w_param: WPARAM,
    pub l_param: LPARAM,
    pub time: DWORD,
    pub pt: POINT,
}

pub type WNDPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;

#[repr(C)]
pub struct WNDCLASSW {
    pub style: UINT,
    pub lpfn_wnd_proc: Option<WNDPROC>,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: HINSTANCE,
    pub h_icon: HICON,
    pub h_cursor: HCURSOR,
    pub hbr_background: HBRUSH,
    pub lpsz_menu_name: *const u16,
    pub lpsz_class_name: *const u16,
}

#[link(name = "user32")]
unsafe extern "system" {
    pub fn OpenClipboard(h_wnd_new_owner: HWND) -> BOOL;
    pub fn CloseClipboard() -> BOOL;
    pub fn EmptyClipboard() -> BOOL;
    pub fn IsClipboardFormatAvailable(format: UINT) -> BOOL;
    pub fn GetClipboardData(u_format: UINT) -> HANDLE;
    pub fn SetClipboardData(u_format: UINT, h_mem: HANDLE) -> HANDLE;
    pub fn AddClipboardFormatListener(hwnd: HWND) -> BOOL;
    pub fn RemoveClipboardFormatListener(hwnd: HWND) -> BOOL;
    pub fn RegisterClassW(lp_wnd_class: *const WNDCLASSW) -> ATOM;
    pub fn CreateWindowExW(
        dw_ex_style: DWORD,
        lp_class_name: *const u16,
        lp_window_name: *const u16,
        dw_style: DWORD,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: HWND,
        h_menu: *mut c_void,
        h_instance: HINSTANCE,
        lp_param: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcW(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
    pub fn GetMessageW(
        lp_msg: *mut MSG,
        h_wnd: HWND,
        w_msg_filter_min: UINT,
        w_msg_filter_max: UINT,
    ) -> BOOL;
    pub fn TranslateMessage(lp_msg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lp_msg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(n_exit_code: i32);
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn CreateMutexW(
        lp_mutex_attributes: *mut c_void,
        b_initial_owner: BOOL,
        lp_name: *const u16,
    ) -> HANDLE;
    pub fn GetLastError() -> DWORD;
    pub fn CloseHandle(h_object: HANDLE) -> BOOL;
    pub fn GlobalAlloc(u_flags: UINT, dw_bytes: usize) -> HGLOBAL;
    pub fn GlobalLock(h_mem: HGLOBAL) -> *mut c_void;
    pub fn GlobalUnlock(h_mem: HGLOBAL) -> BOOL;
    pub fn GlobalFree(h_mem: HGLOBAL) -> HGLOBAL;
}

/// Helper to convert a string slice to null-terminated UTF-16
pub fn to_wide_chars(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// Attempts to open clipboard with retry loop (max 6 attempts with 15ms delay).
pub fn safe_open_clipboard(hwnd: HWND) -> bool {
    for _ in 0..6 {
        unsafe {
            if OpenClipboard(hwnd) != FALSE {
                return true;
            }
        }
        sleep(Duration::from_millis(15));
    }
    false
}

/// Reads plain text from clipboard if CF_UNICODETEXT is available.
pub fn read_clipboard_text(hwnd: HWND) -> Option<String> {
    unsafe {
        if IsClipboardFormatAvailable(CF_UNICODETEXT) == FALSE {
            return None;
        }

        if !safe_open_clipboard(hwnd) {
            return None;
        }

        let h_data = GetClipboardData(CF_UNICODETEXT);
        if h_data.is_null() {
            CloseClipboard();
            return None;
        }

        let lock_ptr = GlobalLock(h_data) as *const u16;
        if lock_ptr.is_null() {
            CloseClipboard();
            return None;
        }

        // Calculate length until null terminator
        let mut len = 0;
        while *lock_ptr.add(len) != 0 {
            len += 1;
        }

        let slice = std::slice::from_raw_parts(lock_ptr, len);
        let result = String::from_utf16_lossy(slice);

        GlobalUnlock(h_data);
        CloseClipboard();

        Some(result)
    }
}

/// Writes Unicode text to the Windows clipboard.
pub fn write_clipboard_text(hwnd: HWND, text: &str) -> bool {
    let wide = to_wide_chars(text);
    let byte_len = wide.len() * std::mem::size_of::<u16>();

    unsafe {
        let h_mem = GlobalAlloc(GMEM_MOVEABLE, byte_len);
        if h_mem.is_null() {
            return false;
        }

        let dest = GlobalLock(h_mem) as *mut u16;
        if dest.is_null() {
            GlobalFree(h_mem);
            return false;
        }

        ptr::copy_nonoverlapping(wide.as_ptr(), dest, wide.len());
        GlobalUnlock(h_mem);

        if !safe_open_clipboard(hwnd) {
            GlobalFree(h_mem);
            return false;
        }

        EmptyClipboard();

        let set_res = SetClipboardData(CF_UNICODETEXT, h_mem);
        CloseClipboard();

        if set_res.is_null() {
            GlobalFree(h_mem);
            return false;
        }

        true
    }
}

/// Ensures only a single instance of cliphancer-svc is running.
pub struct SingleInstanceGuard {
    handle: HANDLE,
}

impl SingleInstanceGuard {
    pub fn try_acquire(unique_name: &str) -> Option<Self> {
        let wide_name = to_wide_chars(unique_name);
        unsafe {
            let handle = CreateMutexW(ptr::null_mut(), TRUE, wide_name.as_ptr());
            if handle.is_null() {
                return None;
            }

            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                return None;
            }

            Some(Self { handle })
        }
    }
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}

#![windows_subsystem = "windows"]

mod cleaner;
mod win32;

use cleaner::clean_text;
use std::cell::RefCell;
use std::ptr;
use win32::*;

thread_local! {
    static LAST_PROCESSED_TEXT: RefCell<Option<String>> = const { RefCell::new(None) };
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLIPBOARDUPDATE => {
            if let Some(current_text) = read_clipboard_text(hwnd) {
                let is_same = LAST_PROCESSED_TEXT.with(|last| {
                    last.borrow().as_deref() == Some(&current_text)
                });

                if !is_same {
                    if let Some(cleaned) = clean_text(&current_text) {
                        LAST_PROCESSED_TEXT.with(|last| {
                            *last.borrow_mut() = Some(cleaned.clone());
                        });
                        write_clipboard_text(hwnd, &cleaned);
                    } else {
                        LAST_PROCESSED_TEXT.with(|last| {
                            *last.borrow_mut() = Some(current_text);
                        });
                    }
                }
            }
            0
        }
        WM_DESTROY => {
            unsafe {
                RemoveClipboardFormatListener(hwnd);
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
    }
}

fn main() {
    // 1. Single Instance Protection: ensure only 1 instance of cliphancer-svc runs
    let _guard = match SingleInstanceGuard::try_acquire("Global\\CliphancerSvc_SingleInstanceMutex")
    {
        Some(guard) => guard,
        None => return, // Another instance is already active
    };

    let class_name = to_wide_chars("CliphancerSvcWindowClass");
    let window_title = to_wide_chars("CliphancerSvc");

    let wnd_class = WNDCLASSW {
        style: 0,
        lpfn_wnd_proc: Some(window_proc),
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance: ptr::null_mut(),
        h_icon: ptr::null_mut(),
        h_cursor: ptr::null_mut(),
        hbr_background: ptr::null_mut(),
        lpsz_menu_name: ptr::null(),
        lpsz_class_name: class_name.as_ptr(),
    };

    unsafe {
        let atom = RegisterClassW(&wnd_class);
        if atom == 0 {
            return;
        }

        // Create message-only window
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_title.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );

        if hwnd.is_null() {
            return;
        }

        // Register to listen for clipboard updates
        if AddClipboardFormatListener(hwnd) == FALSE {
            return;
        }

        // Standard Win32 message loop (blocks on GetMessage, 0% CPU consumption while waiting)
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

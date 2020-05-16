use chrome_native_messaging::event_loop;
use serde_json::Value;
use std::io;
use std::mem;
use std::ptr;
use std::str;
use std::thread;
use winapi::{
    shared::{
        minwindef::{LPARAM, LRESULT, UINT, WPARAM},
        windef::{HBRUSH, HWND},
    },
    um::{
        wingdi::{GetStockObject, WHITE_BRUSH},
        winuser::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FindWindowW,
            GetMessageW, LoadCursorW, LoadIconW, PostQuitMessage, RegisterClassW, SendMessageW,
            ShowWindow, TranslateMessage, UpdateWindow, CS_HREDRAW, CS_VREDRAW, IDC_ARROW,
            IDI_APPLICATION, MSG, SW_NORMAL, WM_CLOSE, WM_CREATE, WM_DESTROY, WNDCLASSW,
            WS_OVERLAPPEDWINDOW,
        },
    },
};

#[link(name = "monkeyhook.dll", kind = "dylib")]
extern "C" {
    fn sethook() -> bool;
    fn unhook() -> bool;
}

fn callback(value: Value) -> io::Result<()> {
    if value.eq("suppressContextMenu") {
        unsafe {
            let hwnd = FindWindowW(
                encode("MozillaDropShadowWindowClass").as_ptr(),
                ptr::null_mut(),
            );
            if !hwnd.is_null() {
                SendMessageW(hwnd, WM_CLOSE, 0, 0);
            }
        }
    }
    Ok(())
}

fn main() {
    unsafe {
        let class_name = encode("monkey_gestures_window_class");
        if !register_wndclass(&class_name) {
            return;
        }

        let hwnd = create_window(&class_name);
        if hwnd.is_null() {
            return;
        }
        ShowWindow(hwnd, SW_NORMAL);
        UpdateWindow(hwnd);
        let mut msg = mem::MaybeUninit::<MSG>::uninit().assume_init();
        loop {
            if GetMessageW(&mut msg, ptr::null_mut(), 0, 0) == 0 {
                return;
            }
            TranslateMessage(&mut msg);
            DispatchMessageW(&mut msg);
        }
    }
}

fn encode(source: &str) -> Vec<u16> {
    source.encode_utf16().chain(Some(0)).collect()
}

unsafe fn register_wndclass(class_name: &[u16]) -> bool {
    let mut winc = mem::zeroed::<WNDCLASSW>();
    winc.style = CS_HREDRAW | CS_VREDRAW;
    winc.lpfnWndProc = Some(win_proc);
    winc.hIcon = LoadIconW(ptr::null_mut(), IDI_APPLICATION);
    winc.hCursor = LoadCursorW(ptr::null_mut(), IDC_ARROW);
    winc.hbrBackground = GetStockObject(WHITE_BRUSH as i32) as HBRUSH;
    winc.lpszClassName = class_name.as_ptr();

    RegisterClassW(&winc) > 0
}

unsafe fn create_window(class_name: &[u16]) -> HWND {
    CreateWindowExW(
        0,
        class_name.as_ptr(),
        encode("Hello, World!").as_ptr(),
        WS_OVERLAPPEDWINDOW,
        0,
        0,
        200,
        200,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
    )
}

unsafe extern "system" fn win_proc(hwnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            thread::spawn(|| event_loop(callback));
            sethook();
        }
        WM_CLOSE => {
            unhook();
            DestroyWindow(hwnd);
        }
        WM_DESTROY => PostQuitMessage(0),
        _ => return DefWindowProcW(hwnd, msg, wp, lp),
    };
    0
}

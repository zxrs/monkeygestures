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
            GetMessageW, LoadCursorW, LoadIconW, PostQuitMessage, RegisterClassW, SendInput,
            SendMessageW, ShowWindow, TranslateMessage, UpdateWindow, CS_HREDRAW, CS_VREDRAW,
            IDC_ARROW, IDI_APPLICATION, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, MSG, SW_NORMAL,
            VK_LEFT, VK_LMENU, VK_RIGHT, WM_CLOSE, WM_CREATE, WM_DESTROY, WNDCLASSW,
            WS_OVERLAPPEDWINDOW,
        },
    },
};

#[link(name = "monkeyhook.dll", kind = "dylib")]
extern "C" {
    fn sethook() -> bool;
    fn unhook() -> bool;
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
        encode("MonkeyGestures").as_ptr(),
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

fn callback(value: Value) -> io::Result<()> {
    match value.as_str() {
        Some("goBack") => unsafe {
            // Alt + <-
            key_down(VK_LMENU);
            key_enter(VK_LEFT);
            key_up(VK_LMENU);
        },
        Some("goForward") => unsafe {
            // Alt + ->
            key_down(VK_LMENU);
            key_enter(VK_RIGHT);
            key_up(VK_LMENU);
        },
        Some("suppressContextMenu") => unsafe {
            let hwnd = get_window("MozillaDropShadowWindowClass")?;
            SendMessageW(hwnd, WM_CLOSE, 0, 0);
        },
        _ => (),
    }
    Ok(())
}

unsafe fn get_window(class_name: &str) -> io::Result<HWND> {
    let class_name_ = encode(class_name);
    let hwnd = FindWindowW(class_name_.as_ptr(), ptr::null_mut());
    if hwnd.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("{} HWND not found.", &class_name),
        ));
    }
    Ok(hwnd)
}

unsafe fn create_input(key_code: i32, flags: u32) -> INPUT {
    let mut input = mem::zeroed::<INPUT>();
    input.type_ = INPUT_KEYBOARD;
    let mut ki = input.u.ki_mut();
    ki.wVk = key_code as u16;
    ki.dwFlags = flags;
    input
}

unsafe fn key_down(key_code: i32) {
    let mut input = create_input(key_code, 0);
    SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
}

unsafe fn key_up(key_code: i32) {
    let mut input = create_input(key_code, KEYEVENTF_KEYUP);
    SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
}

unsafe fn key_enter(key_code: i32) {
    key_down(key_code);
    key_up(key_code);
}

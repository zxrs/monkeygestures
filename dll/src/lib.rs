#![allow(non_upper_case_globals)]

use chrome_native_messaging::write_output;
use serde_json::json;
use std::io;
use std::ptr;
use std::str;
use winapi::{
    ctypes::c_int,
    shared::minwindef::{DWORD, HINSTANCE, LPARAM, LRESULT, WPARAM},
    shared::windef::HHOOK,
    um::winnt::{DLL_PROCESS_ATTACH, LONG, PVOID},
    um::winuser::{
        CallNextHookEx, GetClassNameA, GetKeyState, SetWindowsHookExW, UnhookWindowsHookEx,
        WindowFromPoint, GET_WHEEL_DELTA_WPARAM, HC_ACTION, MSLLHOOKSTRUCT, VK_RBUTTON,
        WH_MOUSE_LL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP,
    },
};

#[link(name = "shareddata", kind = "static")]
extern "C" {
    static mut gHook: HHOOK;
    static mut gLastX: LONG;
    static mut gLastY: LONG;
}
static mut gDll: HINSTANCE = ptr::null_mut();
const TOLERANCE: LONG = 10;
const MOVE_UP: &str = "U";
const MOVE_DOWN: &str = "D";
const MOVE_LEFT: &str = "L";
const MOVE_RIGHT: &str = "R";
const WHEEL_UP: &str = "+";
const WHEEL_DOWN: &str = "-";
const GESTURE_START: &str = "?";
const GESTURE_END: &str = "!";

unsafe extern "system" fn hook_proc(code: c_int, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code == HC_ACTION {
        let mouse = *(lp as *const MSLLHOOKSTRUCT);
        let hwnd = WindowFromPoint(mouse.pt);
        if !hwnd.is_null() {
            let mut buf = [0u8; 64];
            let len = GetClassNameA(hwnd, buf.as_mut_ptr() as *mut i8, 64);
            if let Ok(class_name) = str::from_utf8(&buf[..len as usize]) {
                if class_name.eq("MozillaWindowClass") {
                    match wp as u32 {
                        WM_RBUTTONDOWN => {
                            // Start Gesture
                            gLastX = mouse.pt.x;
                            gLastY = mouse.pt.y;
                            send(GESTURE_START);
                        }
                        WM_MOUSEMOVE => {
                            // Progress Gesture
                            if GetKeyState(VK_RBUTTON) < 0 {
                                let x = mouse.pt.x;
                                let y = mouse.pt.y;
                                let dx = (x - gLastX).abs();
                                let dy = (y - gLastY).abs();

                                if dx > TOLERANCE || dy > TOLERANCE {
                                    if dx > dy {
                                        if x < gLastX {
                                            send(MOVE_LEFT);
                                        } else {
                                            send(MOVE_RIGHT);
                                        }
                                    } else {
                                        if y < gLastY {
                                            send(MOVE_UP);
                                        } else {
                                            send(MOVE_DOWN);
                                        }
                                    }
                                    gLastX = x;
                                    gLastY = y;
                                }
                            }
                        }
                        WM_RBUTTONUP => {
                            // Stop Gesture
                            send(GESTURE_END);
                        }
                        WM_MOUSEWHEEL => {
                            // Wheel Gesture
                            if GetKeyState(VK_RBUTTON) < 0 {
                                if GET_WHEEL_DELTA_WPARAM(mouse.mouseData as usize) > 0 {
                                    send(WHEEL_UP);
                                } else {
                                    send(WHEEL_DOWN);
                                }
                                return 1;
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    return CallNextHookEx(gHook, code, wp, lp);
}

#[no_mangle]
pub extern "C" fn sethook() -> bool {
    unsafe {
        gHook = SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), gDll, 0);
        if gHook.is_null() {
            return false;
        }
        true
    }
}

#[no_mangle]
pub extern "C" fn unhook() -> bool {
    unsafe {
        if !gHook.is_null() {
            return UnhookWindowsHookEx(gHook) > 0;
        }
        false
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(h_instance: HINSTANCE, reason: DWORD, _: PVOID) -> c_int {
    if reason == DLL_PROCESS_ATTACH {
        gDll = h_instance;
    }
    return 1;
}

fn send(direction: &str) {
    let value = json!(direction);
    write_output(io::stdout(), &value).ok();
}

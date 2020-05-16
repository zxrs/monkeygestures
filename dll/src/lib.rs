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
    static mut gDirectionChain: [u8; 32];
}

static mut gDll: HINSTANCE = ptr::null_mut();

const TOLERANCE: LONG = 10;

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
                            direction_chain_clear();
                        }
                        WM_MOUSEMOVE => {
                            // Progress Gesture
                            if GetKeyState(VK_RBUTTON) < 0 {
                                let x = mouse.pt.x;
                                let y = mouse.pt.y;
                                let dx = (x - gLastX).abs();
                                let dy = (y - gLastY).abs();

                                if dx > TOLERANCE || dy > TOLERANCE {
                                    let direction = if dx > dy {
                                        if x < gLastX {
                                            "L"
                                        } else {
                                            "R"
                                        }
                                    } else {
                                        if y < gLastY {
                                            "U"
                                        } else {
                                            "D"
                                        }
                                    };
                                    let last_direction = if direction_chain_is_empty() {
                                        ""
                                    } else {
                                        let len = direction_chain_len();
                                        let last = &gDirectionChain[len - 1..len];
                                        str::from_utf8(last).unwrap_or("")
                                    };
                                    if direction.ne(last_direction) {
                                        direction_chain_append(direction);
                                    }
                                    gLastX = x;
                                    gLastY = y;
                                }
                            }
                        }
                        WM_RBUTTONUP => {
                            // Stop Gesture
                            if !direction_chain_is_empty() {
                                if let Ok(direction) = direction_chain_str() {
                                    if !direction.starts_with("W") {
                                        let value = json!(direction);
                                        write_output(io::stdout(), &value).ok();
                                    }
                                }
                            }
                        }
                        WM_MOUSEWHEEL => {
                            // Wheel Gesture
                            if GetKeyState(VK_RBUTTON) < 0 {
                                if GET_WHEEL_DELTA_WPARAM(mouse.mouseData as usize) > 0 {
                                    direction_chain_assign("W+");
                                } else {
                                    direction_chain_assign("W-");
                                }
                                if let Ok(direction) = direction_chain_str() {
                                    let value = json!(direction);
                                    write_output(io::stdout(), &value).ok();
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

// Helper functions for direction chain
unsafe fn direction_chain_str() -> Result<&'static str, str::Utf8Error> {
    str::from_utf8(&gDirectionChain[..direction_chain_len()])
}

unsafe fn direction_chain_clear() {
    gDirectionChain.iter_mut().for_each(|c| *c = 0);
}

unsafe fn direction_chain_assign(direction: &str) {
    direction_chain_clear();
    direction_chain_append(direction);
}

unsafe fn direction_chain_append(direction: &str) {
    let len = direction_chain_len();
    if direction.len() + len > 32 {
        return;
    }
    let dst_ptr = gDirectionChain.as_mut_ptr().offset(len as isize);
    ptr::copy_nonoverlapping(direction.as_ptr(), dst_ptr, direction.len());
}

unsafe fn direction_chain_len() -> usize {
    gDirectionChain.iter().filter(|&c| c != &0).count()
}

unsafe fn direction_chain_is_empty() -> bool {
    direction_chain_len() == 0
}


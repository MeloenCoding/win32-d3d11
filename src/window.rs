use windows::{
    core::{PCSTR},
    s,
    Win32::{
        Foundation::{BOOL, HMODULE, HWND, LPARAM, LRESULT, POINTS, WPARAM},
        Graphics::Gdi::HBRUSH,
        System::{
            LibraryLoader::GetModuleHandleA,
        },
        UI::{
            Input::KeyboardAndMouse::{ReleaseCapture, SetCapture},
            WindowsAndMessaging::{
                CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA, LoadCursorW,
                MessageBoxExA, PeekMessageA, PostQuitMessage, RegisterClassExA, ShowWindow,
                TranslateMessage, HICON, IDC_ARROW, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, MSG,
                PM_REMOVE, WM_CHAR, WM_CLOSE, WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS,
                WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE,
                WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
                WNDCLASSEXA, WNDCLASS_STYLES, WS_CAPTION, WS_MINIMIZEBOX, WS_SYSMENU,
            },
        },
    },
};

use self::{
    errors::{get_error_desc, FatalErrorBase},
    graphics::Graphics,
    keyboard::Keyboard,
    mouse::Mouse,
};

pub mod errors;
pub mod graphics;
pub mod keyboard;
pub mod message;
pub mod mouse;

/**
    We need some public variables for the wndproc because we can't pass in any other arguments in that function.<br>
    I know public variables are bad but i haven't seen a solution to use variables in [`self::wndproc()`].
*/
pub mod io {
    use super::keyboard::Keyboard;
    use super::mouse::Mouse;

    /// The Mouse state
    pub static mut MOUSE: Mouse = Mouse {
        x: 0,
        y: 0,
        event_queue: vec![],
        left_pressed: false,
        right_pressed: false,
        is_in_window: false,
        wheel_pressed: false,
        wheel_delta_carry: 0,
    };

    /// The keyboard state   
    pub static mut KEYBOARD: Keyboard = Keyboard {
        key_states: vec![],
        key_queue: vec![],
        char_queue: vec![],
        auto_repeat_enabled: false,
    };

    // pub static mut GRAPHICS: Graphics = Graphics::setup();

    /// Width of the window
    pub static mut MAX_MOUSE_X: i16 = 0;
    /// Height of the window
    pub static mut MAX_MOUSE_Y: i16 = 0;
}

/// The Window class which holds every recieved windowEvent and the window data.
pub struct Window<'a> {
    pub instance: HMODULE,
    pub class_name: PCSTR,
    pub atom: u16,
    pub width: i16,
    pub height: i16,
    pub class: WNDCLASSEXA,
    pub hwnd: HWND,
    pub msg_buffer: MSG,
    pub last_result: BOOL,
    pub keyboard: &'a mut Keyboard,
    pub mouse: &'a mut Mouse,
    pub graphics: Graphics,
}

/// Create a message box
pub fn create_message_box(
    lptext: PCSTR,
    utype: MESSAGEBOX_STYLE,
    wlanguageid: u16,
) -> MESSAGEBOX_RESULT {
    let lpcaption: PCSTR = match utype {
        MESSAGEBOX_STYLE(16) => {
            s!("Fatal error")
        }
        _ => {
            s!("Warning")
        }
    };

    return unsafe { MessageBoxExA(HWND::default(), lptext, lpcaption, utype, wlanguageid) };
    /*
        Creates, displays, and operates a message box. The message box contains an application-defined message and title, plus any
        combination of predefined icons and push buttons. The buttons are in the language of the system user interface.

        For more info see: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-messageboxexa
    */
}

impl Window<'_> {
    /// Create a window instance
    pub fn new(
        window_name: &str,
        style: WNDCLASS_STYLES,
        window_width: i16,
        window_height: i16,
    ) -> Window<'static> {
        let mut base_details: String = window_name.to_string();
        base_details.push('\0');
        let class_name: PCSTR = PCSTR::from_raw(base_details.as_ptr());

        /*
            hInstance is the handle to an instance or handle to a module. The
            operating system uses this value to identify the executable or EXE
            when it's loaded in memory.
        */
        let instance: HMODULE = unsafe {
            GetModuleHandleA(None).unwrap_or_else(|_| {
                errors::window::WindowError::new(
                    "Unable to create an hInstance with GetModuleHandle.",
                    None,
                    crate::loc!(),
                );
            })
        };

        /*
            Contains window class information. It is used with the RegisterClassEx
            and GetClassInfoEx functions.
            For more info about the fields of this class:
            https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-wndclassexa
        */

        let class: WNDCLASSEXA = WNDCLASSEXA {
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
            style,
            lpfnWndProc: Some(Self::wndproc),
            hInstance: instance,
            hCursor: unsafe {
                LoadCursorW(None, IDC_ARROW).unwrap_or_else(|_| {
                    errors::window::WindowError::new("Unable to load cursor.", None, crate::loc!())
                })
            },
            lpszClassName: class_name,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: HICON(0isize as _),
            hbrBackground: HBRUSH(0isize as _),
            lpszMenuName: PCSTR(0isize as _),
            hIconSm: HICON(0isize as _),
        };

        /*
            If you register the window class by using RegisterClassExA, the application tells the system that
            the windows of the created class expect messages with text or character parameters to use the ANSI
            character set.

            If the function succeeds, the return value is a class atom that uniquely identifies the class being
            registered. If the function fails, the return value is zero.

            For more info see: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassexa
        */
        let atom: u16 = unsafe { RegisterClassExA(&class) };

        if atom == 0 {
            // check if the registerClass function failed
            panic!("unable to register class");
        }

        /*
            Creates an overlapped, pop-up, or child window. It specifies the window class, window title, window
            style, and (optionally) the initial position and size of the window. The function also specifies
            the window's parent or owner, if any, and the window's menu.

            If the function succeeds, the return value is a handle to the new window. If the function fails, the
            return value is NULL. We can get the error info by calling GetLastError. See GetExitCodes().

            For more info see: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexa
        */
        let hwnd: HWND = unsafe {
            CreateWindowExA(
                windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
                class_name,
                class_name,
                WS_CAPTION | WS_MINIMIZEBOX | WS_SYSMENU,
                200,
                200,
                window_width.into(),
                window_height.into(),
                None,
                None,
                instance,
                None,
            )
        };

        unsafe {
            io::KEYBOARD.reset();
            io::MOUSE.reset();
            io::MAX_MOUSE_X = window_width;
            io::MAX_MOUSE_Y = window_height;
        };

        // return the new Window instance
        Window {
            instance,
            class_name,
            atom,
            class,
            hwnd,
            msg_buffer: MSG::default(),
            last_result: BOOL::default(),
            keyboard: unsafe { &mut io::KEYBOARD },
            mouse: unsafe { &mut io::MOUSE },
            width: window_width,
            height: window_height,
            graphics: Graphics::setup(hwnd),
        }
    }

    pub fn show_window(&self) {
        // Sets the specified window's show state.
        // Check for more info: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
        unsafe {
            ShowWindow(
                self.hwnd,
                windows::Win32::UI::WindowsAndMessaging::SHOW_WINDOW_CMD(1),
            );
        };
    }

    pub fn handle_messages(&mut self) -> Option<usize> {
        while unsafe { PeekMessageA(&mut self.msg_buffer, None, 0, 0, PM_REMOVE).as_bool() } {
            if self.msg_buffer.message == WM_QUIT {
                return Some(self.msg_buffer.wParam.0);
            }
            unsafe { TranslateMessage(&mut self.msg_buffer) };
            unsafe { DispatchMessageA(&mut self.msg_buffer) };
        }

        return None;
    }

    extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        /*
            It is very hard to explain how this works without typing a lot of text so i'll just refer you to
            the great video by ChiliTomatoNoodle (https://youtu.be/UUbXK4G_NCM). It explains how the window
            messages work and how to build a good system around it.

            For more info about wndproc see: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nc-winuser-wndproc
            And for a list with all the messages see: https://wiki.winehq.org/List_Of_Windows_Messages
        */

        unsafe {
            match msg {
                // General window messages
                WM_KILLFOCUS => {
                    io::KEYBOARD.reset();
                }
                WM_CLOSE => {
                    DestroyWindow(hwnd);
                }
                WM_DESTROY => {
                    PostQuitMessage(0);
                }

                // Keyboard messages
                WM_CHAR => {
                    io::KEYBOARD.on_char(wparam.0 as u32);
                }
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    // See https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input#keystroke-message-flags
                    let auto_repeat: bool = (lparam.0 >> 30) & 1 == 1;

                    if auto_repeat {
                        io::KEYBOARD.enable_auto_repeat();
                    }
                    io::KEYBOARD.on_key_press(wparam.0 as u32);
                }
                WM_KEYUP | WM_SYSKEYUP => {
                    io::KEYBOARD.disable_auto_repeat();
                    io::KEYBOARD.on_key_release(wparam.0 as u32);
                }

                // Mouse messages
                WM_MOUSEMOVE => {
                    let points: POINTS = make_points(lparam);
                    if points.x >= 0
                        && points.x <= io::MAX_MOUSE_X
                        && points.y >= 0
                        && points.y <= io::MAX_MOUSE_Y
                    {
                        io::MOUSE.on_mouse_move(points);

                        if !io::MOUSE.is_in_window {
                            SetCapture(hwnd);
                            io::MOUSE.on_mouse_enter();
                        }
                    } else {
                        const MK_LBUTTON: i32 = 0x0001;
                        const MK_MBUTTON: i32 = 0x0010;
                        const MK_RBUTTON: i32 = 0x0002;

                        if (wparam.0 as i32) & (MK_LBUTTON | MK_MBUTTON | MK_RBUTTON) > 0 {
                            io::MOUSE.on_mouse_move(points);
                        } else {
                            ReleaseCapture();
                            io::MOUSE.on_mouse_leave();
                        }
                    }
                }
                WM_LBUTTONDOWN => {
                    io::MOUSE.on_left_press();
                }
                WM_LBUTTONUP => {
                    io::MOUSE.on_left_release();
                }
                WM_RBUTTONUP => {
                    io::MOUSE.on_right_release();
                }
                WM_RBUTTONDOWN => {
                    io::MOUSE.on_right_press();
                }

                WM_MBUTTONDOWN => {
                    io::MOUSE.on_wheel_press();
                }
                WM_MBUTTONUP => {
                    io::MOUSE.on_wheel_release();
                }

                WM_MOUSEWHEEL => {
                    let points: POINTS = make_points(lparam);
                    let delta: i16 = get_wheel_delta_wparam(wparam);
                    io::MOUSE.on_wheel_delta(points.x, points.y, delta);
                }

                _ => {
                    return DefWindowProcA(hwnd, msg, wparam, lparam);
                }
            }
            LRESULT(0)
        }
    }

    pub fn print_exit_codes(&self) {
        return println!(
            "{}",
            get_error_desc(Some(self.last_result), Some(self.msg_buffer))
        );
    }
}

/**
    This function is not in the windows crate so i made it my self. For more info <br>
    see [this](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-makepoints?source=recommendations)
*/
pub fn make_points(lparam: LPARAM) -> POINTS {
    let coords: i32 = lparam.0 as i32;

    let x: i16 = (coords & 0x0000_FFFF) as i16;
    let y: i16 = ((coords & -0x10000) >> 16) as i16;

    return POINTS { x, y };
}

/**
    This function is not in the windows crate so i made it my self. For more info <br>
    see [this](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-mousewheel)
*/
pub fn get_wheel_delta_wparam(wparam: WPARAM) -> i16 {
    let wheel_info: i32 = wparam.0 as i32;

    let delta: i16 = ((wheel_info & -0x10000) >> 16) as i16;

    return delta;
}

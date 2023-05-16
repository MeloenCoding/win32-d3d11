const MAX_BUFFER_SIZE: usize = 16;

#[derive(Debug, Clone)]
pub struct Keyboard {
    /// A map of all the keys represented as 0, 1. 0 means key is up and 1 means key is down
    pub key_states: Vec<u8>,

    /// A FIFO (First In First Out) list of all the recent [KeyEvent]'s.
    /// These events are:
    /// - [WM_KEYUP](windows::Windows::Win32::UI::WindowsAndMessaging::WM_KEYUP)
    /// - [WM_SYSKEYUP](const@windows::Windows::Win32::UI::WindowsAndMessaging::WM_SYSKEYUP)
    /// - [WM_KEYDOWN](const@windows::Windows::Win32::UI::WindowsAndMessaging::WM_KEYDOWN)
    /// - [WM_SYSKEYDOWN](const@windows::Windows::Win32::UI::WindowsAndMessaging::WM_SYSKEYDOWN)
    pub key_queue: Vec<KeyEvent>,

    /// A FIFO (First In First Out) list of all the recent [WM_CHAR][ch] [KeyEvent]'s.
    /// [ch]: windows::Windows::Win32::UI::WindowsAndMessaging::WM_CHAR
    pub char_queue: Vec<char>,

    /// If the user keeps a key pressed in this is true.
    pub auto_repeat_enabled: bool,
}

#[derive(Debug, Copy, Clone)]
/// A event with info about the [KeyState] and the keycode.
pub struct KeyEvent {
    pub key_state: KeyState,
    pub key_code: u32,
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// ### Possible [KeyState]'s
/// - `Press` = `0`
/// - `Release` = `1`
pub enum KeyState {
    Press,
    Release,
}

impl Keyboard {
    /// Reset buffers and keystate map
    pub fn reset(&mut self) {
        self.key_states = vec![0; 256];
        self.key_queue = vec![];
        self.char_queue = vec![];
    }

    /// Check if key is pressed and remove it from the [KeyEvent] queue.<br>
    /// If you don't want to remove the key, See [key_is_pressed()]
    pub fn key_is_pressed_pop(&mut self, target_key: u16) -> bool {
        let key_state: bool = self.key_states[target_key as usize] == 1;
        self.key_states[target_key as usize] = 0;
        return key_state;
    }

    /// Check if key is pressed but don't remove it from the KeyEvent queue. See [key_is_pressed_pop()]
    #[allow(dead_code)]
    pub fn key_is_pressed(&self, target_key: u16) -> bool {
        return self.key_states[target_key as usize] == 1;
    }

    /// Get the [KeyEvent] from the [KeyEvent]
    #[allow(dead_code)]
    pub fn read_key(&mut self) -> Option<KeyEvent> {
        if !self.key_queue.is_empty() {
            let e: KeyEvent = self.key_queue.last().unwrap().to_owned();
            self.key_queue.remove(0);
            return Some(e);
        }
        return None;
    }

    /// Read [char] from the [Keyboard.char_queue] and remove it
    pub fn read_char(&mut self) -> Option<char> {
        if !self.char_queue.is_empty() {
            let ch: char = self.char_queue.last().unwrap().to_owned();
            self.char_queue.remove(0);
            return Some(ch);
        }
        return None;
    }

    #[allow(dead_code)]
    pub fn clear_key_queue(&mut self) {
        self.key_queue = vec![];
    }

    #[allow(dead_code)]
    pub fn clear_char_queue(&mut self) {
        self.char_queue = vec![];
    }

    #[allow(dead_code)]
    pub fn clear_all_queues(&mut self) {
        self.clear_char_queue();
        self.clear_key_queue();
    }

    pub fn disable_auto_repeat(&mut self) {
        self.auto_repeat_enabled = false;
    }

    pub fn enable_auto_repeat(&mut self) {
        self.auto_repeat_enabled = true;
    }

    pub fn on_key_press(&mut self, key_code: u32) {
        self.key_states[key_code as usize] = 1;
        self.key_queue.push(KeyEvent {
            key_state: KeyState::Press,
            key_code,
        });
        trim_buffer(&mut self.key_queue);
    }

    pub fn on_key_release(&mut self, key_code: u32) {
        self.key_states[key_code as usize] = 0;
        self.key_queue.push(KeyEvent {
            key_state: KeyState::Release,
            key_code,
        });
        trim_buffer(&mut self.key_queue.as_mut());
    }

    pub fn on_char(&mut self, char_code: u32) {
        self.char_queue.push(char::from_u32(char_code).unwrap());
        trim_buffer(&mut self.char_queue.as_mut());
    }
}

fn trim_buffer<T>(buffer: &mut Vec<T>) {
    while buffer.len() > MAX_BUFFER_SIZE {
        buffer.remove(0);
    }
}

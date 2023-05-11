use std::time::SystemTime;

use windows::Win32::UI::{Input::KeyboardAndMouse::VK_RETURN, WindowsAndMessaging::CS_OWNDC};

use crate::window::Window;

pub struct App<'a> {
    pub window: Window<'a>,
    input_buffer: String,
    time_buffer: SystemTime,
    debug: bool,
    clock_count: u128,
    fps: Fps
}

struct Fps {
    high: u128,
    total: u128,
    low: u128
}

impl App<'_> {
    pub fn create() -> App<'static> {
        let app = App {
            window: Window::new("Example App", CS_OWNDC, 1000, 750),
            input_buffer: String::new(),
            time_buffer: SystemTime::now(),
            debug: true,
            clock_count: 0,
            fps: Fps { high: 0, total: 0, low: u128::MAX }
        };
        app.window.show_window();
        return app;
    }

    pub fn launch(&mut self) -> usize {
        let mut exit_code: Option<usize>;
        self.time_buffer = SystemTime::now();
        loop {
            exit_code = self.window.handle_messages();
            if exit_code.is_some() {
                break;
            }
            self.render_frame();

        }
        self.print_fps_stats();
        return exit_code.unwrap();
    }

    pub fn render_frame(&mut self) {
        // App logic
        if let Some(ch) = self.window.keyboard.read_char() {
            self.input_buffer.push(ch);
        }   

        if self.window.keyboard.key_is_pressed_pop(VK_RETURN.0) {
            println!("{:?}", self.input_buffer);
            self.input_buffer = "".to_string();
        }

        self.window.graphics.end_frame();
        self.calc_fps();
    }

    fn calc_fps(&mut self) {
        if !self.debug { return }
        let time_alive: std::time::Duration = SystemTime::now().duration_since(self.time_buffer).unwrap();

        let frame_time = time_alive.as_micros();

        let cur = 1_000_000 / frame_time;
        if self.fps.high < cur {
            self.fps.high = cur;
        }
        if self.fps.low > cur {
            self.fps.low = cur;
        }
        println!("fps: {cur}");
        
        self.fps.total += cur;
        self.clock_count += 1;
        self.time_buffer = SystemTime::now();
    }

    fn print_fps_stats(&self) {
        if !self.debug { return }
        println!("fps highest: {}", self.fps.high);
        println!("fps lowest: {}", self.fps.low);
        println!("fps avg: {}", self.fps.total / self.clock_count );
    }
}

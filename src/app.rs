use std::time::SystemTime;

use windows::Win32::UI::{
    Input::KeyboardAndMouse::VK_RETURN,
    WindowsAndMessaging::CS_OWNDC,
};

use crate::window::Window;

pub struct App<'a> {
    pub window: Window<'a>,
    input_buffer: String,
    time_buffer: SystemTime,
    start_time_buffer: SystemTime,
    debug: bool,
    clock_count: u128,
    fps: Fps,
    perf_counter: i64,
}

struct Fps {
    high: u128,
    total: u128,
    low: u128,
}

impl App<'_> {
    pub fn create(width: i16, height: i16) -> App<'static> {
        let debug = true;
        let app = App {
            window: Window::new("Example App", CS_OWNDC, width, height, debug),
            input_buffer: String::new(),
            time_buffer: SystemTime::now(),
            start_time_buffer: SystemTime::now(),
            debug,
            clock_count: 0,
            fps: Fps {
                high: 0,
                total: 0,
                low: u128::MAX,
            },
            perf_counter: 0,
        };
        app.window.show_window();
        return app;
    }

    pub fn launch(&mut self) -> usize {
        let mut exit_code: Option<usize>;
        // unsafe { windows::Win32::System::Performance::QueryPerformanceCounter(&mut self.perf_counter) };
        self.time_buffer = SystemTime::now();

        println!("{:#?}", self.window.width);
        println!("{:#?}", self.window.height);

        loop {
            exit_code = self.window.handle_messages();
            if exit_code.is_some() {
                break;
            }
            self.render_frame();
        }

        if self.debug {
            self.print_fps_stats();

            println!("{}", self.perf_counter);

            // Example DirectX error
            // unsafe { self.window.graphics.dx_info_manager.as_mut().unwrap().info_queue.AddMessage(windows::Win32::Graphics::Dxgi::DXGI_DEBUG_APP,
            //     windows::Win32::Graphics::Dxgi::DXGI_INFO_QUEUE_MESSAGE_CATEGORY_INITIALIZATION, windows::Win32::Graphics::Dxgi::DXGI_INFO_QUEUE_MESSAGE_SEVERITY_ERROR,
            //     69, windows::core::PCSTR::from_raw("Test Error message\0".as_ptr()))
            // }.unwrap_or_else(|e| {
            //     crate::window::errors::graphics::HResultError::new(e.code(), crate::loc!(), &e.message().to_string())
            // });

            for msg in self
                .window
                .graphics
                .dx_info_manager
                .as_ref()
                .unwrap()
                .get_messages()
            {
                println!("{}", msg);
            }
        }

        return exit_code.unwrap();
    }

    pub fn render_frame(&mut self) {
        // Test
        // let angle: f32 = 70.0;
        let angle: f32 = SystemTime::now().duration_since(self.start_time_buffer).unwrap().as_secs_f32();
        let mouse_pos = self.window.mouse.get_pos();

        self.window.graphics.clear_buffer([0.0; 4]);
        self.window.graphics.test_triangle(70.0, 0.0, 0.0);
        self.window.graphics.test_triangle(
            angle, 
            mouse_pos.x as f32 / (self.window.width as f32 / 2.0) - 1.0, 
            -(mouse_pos.y as f32 / (self.window.height as f32 / 2.0) - 1.0)
        );

        // App logic
        if let Some(ch) = self.window.keyboard.read_char() {
            self.input_buffer.push(ch);
        }

        if self.window.keyboard.key_is_pressed_pop(VK_RETURN.0) {
            println!("{:?}", self.input_buffer);
            self.input_buffer = "".to_string();
        }

        // Draw screen
        self.window.graphics.end_frame();

        // Debug
        if self.debug {
            self.calc_fps();
        }
    }

    fn calc_fps(&mut self) {
        let time_alive: std::time::Duration =
            SystemTime::now().duration_since(self.time_buffer).unwrap();

        let frame_time = time_alive.as_micros();

        let cur = 1_000_000 / frame_time;

        if self.fps.high < cur {
            self.fps.high = cur;
        }
        if self.fps.low > cur {
            self.fps.low = cur;
        }

        // println!("fps: {cur}");

        self.fps.total += cur;
        self.clock_count += 1;
        self.time_buffer = SystemTime::now();
    }

    fn print_fps_stats(&self) {
        if !self.debug {
            return;
        }
        println!("fps highest: {}", self.fps.high);
        println!("fps lowest: {}", self.fps.low);
        println!("fps avg: {}", self.fps.total / self.clock_count);
    }
}

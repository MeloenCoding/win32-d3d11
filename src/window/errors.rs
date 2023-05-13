pub mod graphics;
pub mod window;

#[derive(Debug)]
pub struct CallLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[macro_export]
macro_rules! loc {
    () => {
        crate::window::errors::CallLocation {
            file: file!().to_string(),
            line: line!(),
            column: column!(),
        }
    };
}

pub trait ErrorBase {
    fn new(error_details: &str, error_code: Option<i32>, origin: CallLocation) -> ! {
        let base_details: String = format!(
            "Error in {}:{}\n{}\0",
            origin.file, origin.line, error_details
        );
        
        let formatted_details: windows::core::PCSTR = windows::core::PCSTR::from_raw(base_details.as_ptr());

        crate::window::create_message_box(
            formatted_details, 
            windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR | windows::Win32::UI::WindowsAndMessaging::MB_OK, 
            0
        );

        std::process::exit(error_code.unwrap_or(1));
    }
}

use super::graphics::Graphics;

pub mod dx_info_module;
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

pub trait FatalErrorBase {
    fn new(
        error_details: &str,
        error_code: Option<i32>,
        origin: CallLocation,
        graphics: Option<&Graphics>,
    ) -> ! {
        let base_details: String = format!(
            "Error in {}:{}\n{}\0",
            origin.file, origin.line, error_details
        );

        let formatted_details: windows::core::PCSTR =
            windows::core::PCSTR::from_raw(base_details.as_ptr());

        crate::window::create_message_box(
            formatted_details,
            windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR
                | windows::Win32::UI::WindowsAndMessaging::MB_OK,
            0,
        );

        if graphics.is_some() {
            for msg in graphics
                .unwrap()
                .dx_info_manager
                .as_ref()
                .unwrap_or_else(|| { 
                    println!("For an error log you should enable debug mode."); 
                    std::process::exit(error_code.unwrap_or(1));
                })
                .get_messages()
            {
                println!("{:#?}", msg);
            }
        }

        std::process::exit(error_code.unwrap_or(1));
    }
}

pub fn get_error_desc(
    last_result: Option<windows::Win32::Foundation::BOOL>,
    msg_buffer: Option<windows::Win32::UI::WindowsAndMessaging::MSG>,
) -> String {
    let err_code: u32 = unsafe { windows::Win32::Foundation::GetLastError().0 }; // Get the last WIN32_ERROR and get the id from it (u32)
    let mut err_buffer: *mut u8 = std::ptr::null_mut(); // Create a buffer for windows where it should store the error message
    if err_code == 0 {
        // If the error code == 0, there is no error. So there is no need for priting a succes error :)
        return format!(
            "Succesfull exit with codes: last getResult: {:?}, wParam: {}",
            last_result.unwrap().0,
            msg_buffer.unwrap().wParam.0
        );
    }

    let err_msg_lenght: u32 = unsafe {
        windows::Win32::System::Diagnostics::Debug::FormatMessageA(
            /*
                Formats a message string. The function requires a message definition as input.

                The function finds the message definition in a message table resource based on
                a message identifier (HRESULT/GetLastError()) and a language identifier (LCID). The function copies the
                formatted message text to an output buffer, processing any embedded insert
                sequences if requested.

                For more info see: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessage
            */
            windows::Win32::System::Diagnostics::Debug::FORMAT_MESSAGE_FROM_SYSTEM | // Use system message tables to retrieve error text
            windows::Win32::System::Diagnostics::Debug::FORMAT_MESSAGE_ALLOCATE_BUFFER, // Allocate buffer on local heap for error text
            None, // Location of the message definition. We use the systems error table so it has to be None
            err_code, // The Errorcode you want a description about
            0,    // LCID (language code identifier) ->
            /*
                This one is a bit weird. In the description the FormatMessage
                function it says we need an LANGID but there is nothing like that in the windows crate. This crate uses a LCID.
                0 means that it will use your system languague. 1033 means US.
                For more info see: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-lcid/70feba9f-294e-491e-b6eb-56532684c37f
            */
            windows::core::PSTR(&mut err_buffer as *mut _ as *mut _), // Man... this took me ages to get working. ->
            /*
                A pointer to a buffer that receives the null-terminated string that specifies the formatted message.
                This buffer cannot be larger than 64K bytes.
                ---
                We first create a mutable null pointer and set the type to a u8 like this:
                let mut err_buffer: *mut u8 = std::ptr::null_mut();
                Then we use the PSTR constructor to create a pointer to a null-terminated string of 8-bit Windows (ANSI) characters.
                like this:
                    PSTR();
                Then we put in a mutable reference to the error_buffer and cast it to an mutable pointer (I have no clue how and why this works)
                    PSTR(&mut err_buffer as *mut _ as *mut _);
            */
            0,
            /*
                If the FORMAT_MESSAGE_ALLOCATE_BUFFER flag is not set, this parameter specifies the size of the output buffer, in TCHARs. If
                FORMAT_MESSAGE_ALLOCATE_BUFFER is set, this parameter specifies the minimum number of TCHARs to allocate for an output buffer.
            */
            None,
            /*
                An array of values that are used as insert values in the formatted message.
                For more info see: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessage#parameters
            */
        )
    };

    if err_msg_lenght == 0 {
        /*
            If the message buffer is empty, there is no available error description
            Could be caused by an invalid error code or an invalid or not correctly installed LCID
        */

        return format!("Code: {}: Unable to find error description", err_code);
    }

    /*
        If there is an error, print all the return codes,
        println!("Unsuccesfull exit with codes getResult: {:?}, wParam: {}, lastError: {}", self.last_result.0, self.msg_buffer.wParam.0, unsafe { GetLastError().0 });
        and print out the description of the code
    */
    let slice: Vec<u8> =
        unsafe { std::slice::from_raw_parts(err_buffer, (err_msg_lenght - 2) as _).to_vec() };

    return format!("Code {}: {}", err_code, String::from_utf8(slice).unwrap());
}

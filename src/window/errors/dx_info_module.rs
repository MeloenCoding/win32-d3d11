use windows::Win32::Graphics::Dxgi::{DXGI_DEBUG_ALL, DXGI_INFO_QUEUE_MESSAGE};

use crate::{loc, window::message};

pub struct Manager {
    pub start_index: u64,
    pub info_queue: windows::Win32::Graphics::Dxgi::IDXGIInfoQueue,
}

impl Manager {
    /// Create a new Manager
    pub fn new() -> Manager {
        let info_queue: windows::Win32::Graphics::Dxgi::IDXGIInfoQueue = unsafe {
            windows::Win32::Graphics::Dxgi::DXGIGetDebugInterface1(0).unwrap_or_else(|e| {
                super::graphics::HResultError::new(
                    e.code(),
                    loc!(),
                    "Unable to get debugInterface",
                );
            })
        };

        return Manager {
            start_index: 0,
            info_queue,
        };
    }

    /// Sets msg_index to capture all of the errors that follow after this function is called
    #[allow(dead_code)]
    pub fn mark(&mut self) {
        self.start_index = unsafe { self.info_queue.GetNumStoredMessages(DXGI_DEBUG_ALL) };
    }

    // https://learn.microsoft.com/en-us/windows/win32/api/d3d11sdklayers/ne-d3d11sdklayers-d3d11_message_id ???
    pub fn get_messages(&self) -> Vec<String> {
        let size: u64 = unsafe { self.info_queue.GetNumStoredMessages(DXGI_DEBUG_ALL) };
        let mut messages: Vec<String> = Vec::new();
        for i in self.start_index..size {
            let info_msg_buffer = &mut DXGI_INFO_QUEUE_MESSAGE::default();
            let mut message_length: usize = 0;
            unsafe {
                self.info_queue
                    .GetMessage(DXGI_DEBUG_ALL, i, None, &mut message_length)
                    .unwrap_or_else(|e| {
                        super::graphics::HResultError::new(
                            e.code(),
                            loc!(),
                            &e.message().to_string(),
                        )
                    })
            };

            unsafe {
                self.info_queue
                    .GetMessage(
                        DXGI_DEBUG_ALL,
                        i,
                        Some(info_msg_buffer),
                        &mut message_length,
                    )
                    .unwrap_or_else(|e| {
                        super::graphics::HResultError::new(
                            e.code(),
                            loc!(),
                            &e.message().to_string(),
                        )
                    })
            };

            if !messages.contains(&format!("[MSG_ID] {}", info_msg_buffer.ID)) {
                messages.insert(messages.len(), format!("[MSG_ID] {}", info_msg_buffer.ID));
                messages.insert(
                    messages.len(),
                    format!(
                        "[CATEGORY_ID] {}",
                        message::dx_category_id_to_str(info_msg_buffer.Category)
                            .unwrap_or(info_msg_buffer.Category.0.to_string())
                    ),
                );

                if message_length > 0 {
                    let char_vec = unsafe {
                        std::slice::from_raw_parts(
                            info_msg_buffer.pDescription,
                            info_msg_buffer.DescriptionByteLength,
                        )
                    };

                    messages.insert(
                        messages.len(),
                        format!(
                            "[SEVERITY] {}",
                            message::dx_severity_id_to_str(info_msg_buffer.Severity)
                                .unwrap_or(info_msg_buffer.Severity.0.to_string())
                        ),
                    );

                    messages.insert(
                        messages.len(),
                        format!(
                            "[DESCRIPTION]\n{}",
                            &String::from_utf8(char_vec.to_vec()).unwrap_or(format!(
                                "Message found but is corrupted [msglength: {}]\n{}",
                                info_msg_buffer.DescriptionByteLength,
                                String::from_utf8_lossy(char_vec)
                            ))
                        ),
                    );
                } else {
                    messages.insert(
                        messages.len(),
                        "[DESCRIPTION] No message was found".to_string(),
                    );
                }
                messages.insert(messages.len(), "\n".to_string());
            }
        }
        return messages;
    }
}

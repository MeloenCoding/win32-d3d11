use std::{
    mem::size_of,
};

use windows::Win32::Graphics::Dxgi::{DXGI_DEBUG_ALL, DXGI_INFO_QUEUE_MESSAGE};

use crate::loc;

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

    pub fn get_messages(&self) -> Vec<String> {
        let size: u64 = unsafe { self.info_queue.GetNumStoredMessages(DXGI_DEBUG_ALL) };
        let mut messages: Vec<String> = Vec::new();
        for i in self.start_index..size {
            let mut message_length: usize = 0;
            let info_msg_buffer = &mut DXGI_INFO_QUEUE_MESSAGE::default(); // for some reason this function has to be called before an other function else it won't print the entire error message. (wth)

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

            messages.insert(
                messages.len(),
                format!("[CATEGORY_ID] {}", info_msg_buffer.Category.0),
            );

            if message_length > 0 {
                let b = unsafe {
                    std::slice::from_raw_parts(
                        info_msg_buffer.pDescription,
                        message_length
                            - size_of::<windows::Win32::Graphics::Dxgi::DXGI_INFO_QUEUE_MESSAGE>(),
                    )
                };
                messages.insert(
                    messages.len(),
                    "[DESCRIPTION] ".to_string()
                        + &String::from_utf8(b.to_vec())
                            .unwrap_or("No message was found".to_string()),
                );
            } else {
                messages.insert(0, "[DESCRIPTION] No message was found".to_string())
            }
        }
        return messages;
    }
}

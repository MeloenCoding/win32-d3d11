use crate::loc;
use crate::window::errors;
use crate::window::graphics::FLOAT3;
use windows::Win32::Graphics::Direct3D11::D3D11_BIND_VERTEX_BUFFER;
use windows::Win32::Graphics::Direct3D11::D3D11_CPU_ACCESS_FLAG;
use windows::Win32::Graphics::Direct3D11::D3D11_RESOURCE_MISC_FLAG;
use windows::Win32::Graphics::Direct3D11::D3D11_USAGE_DEFAULT;
use windows::Win32::Graphics::Direct3D11::ID3D11Buffer;
use windows::Win32::Graphics::Direct3D11::D3D11_BUFFER_DESC;
use windows::Win32::Graphics::Direct3D11::D3D11_SUBRESOURCE_DATA;

pub struct VertexBuffer {
    buffer: *mut Option<ID3D11Buffer>,
    buff_desc: D3D11_BUFFER_DESC,
    data: D3D11_SUBRESOURCE_DATA
}

impl VertexBuffer {
    pub fn new(vertices: Vec<FLOAT3>) -> VertexBuffer {
        let vertex = VertexBuffer { 
            buff_desc: D3D11_BUFFER_DESC {
                ByteWidth: (vertices.len() * std::mem::size_of::<FLOAT3>()) as u32,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_VERTEX_BUFFER,
                CPUAccessFlags: D3D11_CPU_ACCESS_FLAG::default(),
                MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
                StructureByteStride: std::mem::size_of::<FLOAT3>() as u32,
            },
            data: D3D11_SUBRESOURCE_DATA {
                pSysMem: vertices.as_ptr() as *const _,
                SysMemPitch: 0,
                SysMemSlicePitch: 0,
            },
            buffer: &mut None
        };

        unsafe {
            &self
                .device
                .CreateBuffer(&buff_desc, Some(&vertex.data), Some(vertex.buffer))
        }
        .as_ref()
        .unwrap_or_else(|e| {
            errors::graphics::GraphicsError::new(
                &e.message().to_string(),
                Some(e.code().0),
                loc!(),
                Some(self),
            )
        });
        return vertex;
    }
}
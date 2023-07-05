use std::slice::from_raw_parts;
use windows::{
    self,
    s,
    Win32::{
        Foundation::{S_OK, TRUE, FALSE},
        Graphics::{
            Direct3D::{
                Fxc::{
                    D3DCompileFromFile, D3DCOMPILE_DEBUG,
                    D3DCOMPILE_SKIP_OPTIMIZATION,
                },
                ID3DBlob, D3D_DRIVER_TYPE_HARDWARE,
                D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            },
            Direct3D11::{
                D3D11CreateDevice, ID3D11Buffer, ID3D11Device,
                ID3D11DeviceContext, ID3D11InputLayout, ID3D11RenderTargetView, ID3D11Resource, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC,
                D3D11_CPU_ACCESS_FLAG, D3D11_CREATE_DEVICE_DEBUG, D3D11_INPUT_ELEMENT_DESC,
                D3D11_INPUT_PER_VERTEX_DATA,
                D3D11_RESOURCE_MISC_FLAG, D3D11_SDK_VERSION, D3D11_SUBRESOURCE_DATA,
                D3D11_USAGE_DEFAULT, D3D11_VIEWPORT, D3D11_APPEND_ALIGNED_ELEMENT, D3D11_BIND_INDEX_BUFFER, D3D11_USAGE_DYNAMIC, D3D11_BIND_CONSTANT_BUFFER, D3D11_CPU_ACCESS_WRITE, D3D11_CPU_ACCESS_READ, D3D11_DEPTH_STENCIL_DESC, D3D11_DEPTH_WRITE_MASK, D3D11_DEPTH_WRITE_MASK_ALL, D3D11_COMPARISON_LESS, D3D11_DEPTH_STENCILOP_DESC, ID3D11DepthStencilState, D3D11_TEXTURE2D_DESC, D3D11_BIND_DEPTH_STENCIL, ID3D11Texture2D, ID3D11DepthStencilView, D3D11_DEPTH_STENCIL_VIEW_DESC, D3D11_DSV_DIMENSION_TEXTURE2D, D3D11_DEPTH_STENCIL_VIEW_DESC_0, D3D11_TEX2D_DSV, D3D11_CLEAR_DEPTH,
            },
            Dxgi::{
                Common::{
                    DXGI_ALPHA_MODE_UNSPECIFIED, DXGI_FORMAT_R32G32B32_FLOAT,
                    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC, DXGI_FORMAT_R16_UINT, DXGI_FORMAT_D32_FLOAT,
                },
                CreateDXGIFactory2, IDXGIFactory4, IDXGISwapChain1, DXGI_ERROR_DEVICE_REMOVED, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
    },
};

use directx_math::*;

use crate::loc;

use super::errors::{self, dx_info_module::Manager, FatalErrorBase};

pub struct Graphics {
    pub dx_info_manager: Option<crate::window::errors::dx_info_module::Manager>,
    pub resources: Option<Resources>,
    dxgi_factory: IDXGIFactory4,
    device: ID3D11Device,
    window_width: i16,
    window_height: i16,
}

pub struct Resources {
    pub swap_chain: IDXGISwapChain1,
    pub context: ID3D11DeviceContext,
    pub target: ID3D11RenderTargetView,
    pub depth_stencil_view: ID3D11DepthStencilView
}

pub struct VECTOR3 {
    x: f32,
    y: f32,
    z: f32,
}

pub struct RGBA {
    r: f32,
    g: f32,
    b: f32,
    a: f32
}

pub struct CB2 {
    face_colors: [RGBA; 6]
}

const _RGBA_NORM: f32 = 1.0 / 255.0;

impl Graphics {
    pub fn setup(
        hwnd: windows::Win32::Foundation::HWND,
        debug: bool,
        window_height: i16,
        window_width: i16,
    ) -> Graphics {
        let (dxgi_factory, device) = Graphics::create_device();
        let mut dx_info_manager = None;

        if debug {
            dx_info_manager = Some(Manager::new());
        }

        let mut graphics = Graphics {
            dxgi_factory,
            device,
            window_height,
            window_width,
            resources: None,
            dx_info_manager,
        };

        graphics.bind_to_window(&hwnd);

        return graphics;
    }

    pub fn end_frame(&self) {
        let h_result = unsafe { self.resources.as_ref().unwrap().swap_chain.Present(1, 0) };
        if h_result == DXGI_ERROR_DEVICE_REMOVED {
            let reason = unsafe { self.device.GetDeviceRemovedReason().unwrap_err() };
            errors::graphics::DeviceRemovedError::new(
                "DXGI_ERROR_DEVICE_REMOVED",
                Some(reason.code().0),
                loc!(),
                Some(self),
            )
        } else if h_result != S_OK {
            errors::graphics::HResultError::new(
                h_result,
                loc!(),
                "Presenting scene to swapchain failed",
            );
        }
    }

    fn rgba_norm(r: u8, g: u8, b: u8, a: f32) -> [f32; 4] {
        return [
            r as f32 * _RGBA_NORM,
            g as f32 * _RGBA_NORM,
            b as f32 * _RGBA_NORM,
            a,
        ];
    }

    
    pub fn clear_buffer(&self, rgba: [f32; 4]) {
        let context = &self.resources.as_ref().unwrap().context;
        unsafe {
            context.ClearRenderTargetView(&self.resources.as_ref().unwrap().target, &rgba[0]);
            context.ClearDepthStencilView(&self.resources.as_ref().unwrap().depth_stencil_view, D3D11_CLEAR_DEPTH.0, 1.0, 0);
        };
    }
        
    pub fn draw_sample_text(&self) {
        // self.
    }

    pub fn test_triangle(&self, angle: f32, x: f32, z:f32) {
    // pub fn test_triangle(&self, angle: f32) {
        let context = &self.resources.as_ref().unwrap().context;

        // Please ignore :) i hate it as much as you do... and im lazy
        let shader_path = std::env::current_exe().ok().unwrap().parent().unwrap().parent().unwrap().parent().unwrap().join("src\\window\\graphics\\shaders"); 

        // Create cube vertex's
        let vertices: Vec<VECTOR3> = vec![
            VECTOR3 {x: -1.0, y: -1.0, z: -1.0},
            VECTOR3 {x: 1.0, y: -1.0, z: -1.0 },
            
            VECTOR3 {x: -1.0, y: 1.0, z: -1.0},
            VECTOR3 {x: 1.0, y: 1.0, z: -1.0},
            
            VECTOR3 {x: -1.0, y: -1.0, z: 1.0},
            VECTOR3 {x: 1.0, y: -1.0, z: 1.0},

            VECTOR3 {x: -1.0, y: 1.0, z: 1.0},
            VECTOR3 {x: 1.0, y: 1.0, z: 1.0}, 
        ];

        // let compile_flags = D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION;
        let compile_flags = if cfg!(debug_assertions) {
            D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
            // 0
        } else {
            0
        };

        // Create a vertex buffer
        let vertex_buff: *mut Option<ID3D11Buffer> = &mut None;

        let buff_desc: D3D11_BUFFER_DESC = D3D11_BUFFER_DESC {
            ByteWidth: (vertices.len() * std::mem::size_of::<VECTOR3>()) as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_VERTEX_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_FLAG::default(),
            MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
            StructureByteStride: std::mem::size_of::<VECTOR3>() as u32,
        };

        let data: D3D11_SUBRESOURCE_DATA = D3D11_SUBRESOURCE_DATA {
            pSysMem: vertices.as_ptr() as *const _,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        unsafe {
            &self
                .device
                .CreateBuffer(&buff_desc, Some(&data), Some(vertex_buff))
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

        // Create index buffer
        let indices: Vec<u16> = vec![
            0,2,1, 2,3,1,
            1,3,5, 3,7,5,
            2,6,3, 3,6,7,
            4,5,7, 4,7,6,
            0,4,2, 2,4,6,
            0,1,4, 1,5,4
        ];

        let index_buffer: *mut Option<ID3D11Buffer> = &mut None;

        let indices_buff_desc: D3D11_BUFFER_DESC = D3D11_BUFFER_DESC {
            ByteWidth: (indices.len() * std::mem::size_of::<VECTOR3>()) as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_INDEX_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_FLAG::default(),
            MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
            StructureByteStride: std::mem::size_of::<VECTOR3>() as u32,
        };

        let indices_data: D3D11_SUBRESOURCE_DATA = D3D11_SUBRESOURCE_DATA {
            pSysMem: indices.as_ptr() as *const _,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        unsafe {
            &self
                .device
                .CreateBuffer(&indices_buff_desc, Some(&indices_data), Some(index_buffer))
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

        unsafe { context.IASetIndexBuffer((*index_buffer).as_ref().unwrap(), DXGI_FORMAT_R16_UINT, 0) };

        // Create VertexBuffer on the Input Assembler (IA) [see](https://learn.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-graphics-pipeline)
        let mut stride = std::mem::size_of::<VECTOR3>() as u32;
        let mut offset = 0u32;

        unsafe {
            context.IASetVertexBuffers(
                0,
                1,
                Some(vertex_buff),
                Some(&mut stride),
                Some(&mut offset),
            )
        };

        // Compile vertex shaders
        let mut vertex_shader = None;
        let mut vertex_blob: Option<ID3DBlob> = None;
        let vertex_file = shader_path
            .join("VertexShader.hlsl")
            .to_str()
            .unwrap()
            .to_string();

        unsafe {
            D3DCompileFromFile(
                &format!("{}\0", vertex_file).into(),
                None,
                None,
                s!("main"),
                s!("vs_5_0"),
                compile_flags,
                0,
                &mut vertex_blob,
                None,
            )
            .unwrap_or_else(|e| {
                errors::graphics::GraphicsError::new(
                    &e.message().to_string(),
                    Some(e.code().0),
                    loc!(),
                    Some(self),
                )
            })
        };

        unsafe {
            &self.device.CreateVertexShader(
                std::slice::from_raw_parts(
                    vertex_blob.as_ref().unwrap().GetBufferPointer().cast(),
                    vertex_blob.as_ref().unwrap().GetBufferSize(),
                ),
                None,
                Some(&mut vertex_shader),
            )
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

        // And set them on the Vertex Stage (VS) [see](https://learn.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-graphics-pipeline)
        unsafe { context.VSSetShader(&vertex_shader.unwrap(), None) };
        
        let const_buff: *mut Option<ID3D11Buffer> = &mut None;
        
        let vs_const_buff = {
            XMMatrixTranspose(
                (
                    XMMatrix(XMMatrixRotationZ(angle)) *
                    XMMatrix(XMMatrixRotationX(angle)) *
                    XMMatrix(XMMatrixTranslation(x, 0.0, z + 4.0)) *
                    XMMatrix(XMMatrixPerspectiveLH(1.0, 3.0/4.0, 0.5, 10.0))
                ).0
            )
        };     

        let const_subresource_data: D3D11_SUBRESOURCE_DATA = D3D11_SUBRESOURCE_DATA {
            pSysMem: unsafe { vs_const_buff.r.as_ptr() } as *const _,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        let const_buff_desc: D3D11_BUFFER_DESC = D3D11_BUFFER_DESC {
            ByteWidth: std::mem::size_of_val(&vs_const_buff) as u32,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
            MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
            StructureByteStride: 0,
        };

        unsafe {
            &self
                .device
                .CreateBuffer(&const_buff_desc, Some(&const_subresource_data), Some(const_buff))
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

        unsafe { context.VSSetConstantBuffers(0, Some(std::slice::from_raw_parts(const_buff, 1))) }

        let const_buff2: *mut Option<ID3D11Buffer> = &mut None;   

        const cb2: CB2 = CB2 { 
            face_colors: [
                RGBA { r: 1.0, g: 0.0, b: 1.0, a: 1.0 },
                RGBA { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
                RGBA { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
                RGBA { r: 0.0, g: 0.0, b: 1.0, a: 0.0 },
                RGBA { r: 1.0, g: 1.0, b: 0.0, a: 0.0 },
                RGBA { r: 0.0, g: 1.0, b: 1.0, a: 0.0 },
            ]
        };

        let const_subresource_data2: D3D11_SUBRESOURCE_DATA = D3D11_SUBRESOURCE_DATA {
            pSysMem: (cb2.face_colors.as_ptr()) as *const _,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        let const_buff_desc2: D3D11_BUFFER_DESC = D3D11_BUFFER_DESC {
            ByteWidth: std::mem::size_of::<CB2>() as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_FLAG::default(),
            MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
            StructureByteStride: 0,
        };

        unsafe {
            &self
                .device
                .CreateBuffer(&const_buff_desc2, Some(&const_subresource_data2), Some(const_buff2))
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

        unsafe { context.PSSetConstantBuffers(0, Some(std::slice::from_raw_parts(const_buff2, 1))) }

        // Same for the pixel shaders
        let pixel_file = shader_path
            .join("PixelShader.hlsl")
            .to_str()
            .unwrap()
            .to_string();

        let mut pixel_shader = None;
        let mut pixel_blob = None;

        unsafe {
            D3DCompileFromFile(
                &format!("{}\0", pixel_file).into(),
                None,
                None,
                s!("main"),
                s!("ps_5_0"),
                compile_flags,
                0,
                &mut pixel_blob,
                None,
            )
        }
        .unwrap_or_else(|e| {
            errors::graphics::GraphicsError::new(
                &e.message().to_string(),
                Some(e.code().0),
                loc!(),
                Some(self),
            )
        });

        unsafe {
            &self.device.CreatePixelShader(
                std::slice::from_raw_parts(
                    pixel_blob.as_ref().unwrap().GetBufferPointer().cast(),
                    pixel_blob.as_ref().unwrap().GetBufferSize(),
                ),
                None,
                Some(&mut pixel_shader),
            )
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

        // And set them on the Pixel Shader (PS) [see] (https://learn.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-graphics-pipeline)
        unsafe { context.PSSetShader(&pixel_shader.unwrap(), None) };

        // Bind to render target
        let rendertarget = Some(Some((&self.resources.as_ref().unwrap().target).to_owned()));
        let pprendertargetviews = rendertarget.as_ref().map(core::slice::from_ref);
        unsafe { context.OMSetRenderTargets(pprendertargetviews, Some(&self.resources.as_ref().unwrap().depth_stencil_view)) };

        // Configure viewport
        let view_port: D3D11_VIEWPORT = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: self.window_width as f32,
            Height: self.window_height as f32 - 40.0,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };

        unsafe {
            context.RSSetViewports(Some(
                Some(view_port).as_ref().map(core::slice::from_ref).unwrap(),
            ))
        };

        unsafe { context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST) };

        let pos_element_desc = D3D11_INPUT_ELEMENT_DESC {
            SemanticName: s!("Position"),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT, 
            InputSlot: 0,
            AlignedByteOffset: D3D11_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        };

        let input_element_desc: Vec<D3D11_INPUT_ELEMENT_DESC> = vec![pos_element_desc];

        let input_layout: Option<*mut Option<ID3D11InputLayout>> = Some(&mut None);
        unsafe {
            self.device.CreateInputLayout(
                &input_element_desc,
                from_raw_parts(
                    vertex_blob.as_ref().unwrap().GetBufferPointer().cast(),
                    vertex_blob.as_ref().unwrap().GetBufferSize(),
                ),
                input_layout,
            )
        }
        .unwrap_or_else(|e| {
            errors::graphics::GraphicsError::new(
                &e.message().to_string(),
                Some(e.code().0),
                loc!(),
                Some(self),
            )
        });

        unsafe { context.IASetInputLayout((*input_layout.unwrap()).as_ref().unwrap()) };

        unsafe { context.DrawIndexed(indices.len().try_into().unwrap(), 0, 0) };
    }

    fn bind_to_window(&mut self, hwnd: &windows::Win32::Foundation::HWND) {
        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: self.window_width as u32,
            Height: self.window_height as u32,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            Stereo: windows::Win32::Foundation::BOOL(0),
            AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
            Flags: 0,
            Scaling: DXGI_SCALING_STRETCH,
        };

        let swap_chain: IDXGISwapChain1 = unsafe {
            self.dxgi_factory
                .CreateSwapChainForHwnd(&self.device, *hwnd, &swap_chain_desc, None, None)
                .unwrap_or_else(|e| {
                    errors::graphics::GraphicsError::new(
                        &e.message().to_string(),
                        Some(e.code().0),
                        loc!(),
                        Some(self),
                    );
                })
        };

        let context = unsafe { self.device.GetImmediateContext() }.unwrap_or_else(|e| {
            errors::graphics::GraphicsError::new(
                &e.message().to_string(),
                Some(e.code().0),
                loc!(),
                Some(self),
            )
        });

        let back_buffer: ID3D11Resource = unsafe {
            swap_chain.GetBuffer(0).unwrap_or_else(|e| {
                errors::graphics::GraphicsError::new(
                    &e.message().to_string(),
                    Some(e.code().0),
                    loc!(),
                    Some(self),
                )
            })
        };

        let mut target: Option<ID3D11RenderTargetView> = None;
        unsafe {
            self.device
                .CreateRenderTargetView(&back_buffer, None, Some(&mut target))
                .unwrap_or_else(|e| {
                    errors::graphics::GraphicsError::new(
                        &e.message().to_string(),
                        Some(e.code().0),
                        loc!(),
                        Some(self),
                    )
                })
        };
        
        let mut ds_state: Option<ID3D11DepthStencilState> = None;

        let ds_desc: D3D11_DEPTH_STENCIL_DESC = D3D11_DEPTH_STENCIL_DESC { 
            DepthEnable: TRUE,
            DepthWriteMask: D3D11_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D11_COMPARISON_LESS,
            StencilEnable: FALSE,
            StencilReadMask: 0,
            StencilWriteMask: 0,
            FrontFace: D3D11_DEPTH_STENCILOP_DESC::default(),
            BackFace: D3D11_DEPTH_STENCILOP_DESC::default()
        }; 

        unsafe { 
            self.device.CreateDepthStencilState(&ds_desc, Some(&mut ds_state)) 
            .unwrap_or_else(|e| {
                errors::graphics::GraphicsError::new(
                    &e.message().to_string(),
                    Some(e.code().0),
                    loc!(),
                    Some(self),
                )
            });

            context.OMSetDepthStencilState(&ds_state.unwrap(), 1)
        };


        let mut depth_stencil: Option<ID3D11Texture2D> = None;
        let desc_depth: D3D11_TEXTURE2D_DESC = D3D11_TEXTURE2D_DESC { 
            Width: self.window_width as u32, 
            Height: self.window_height as u32, 
            MipLevels: 1, 
            ArraySize: 1, 
            Format: DXGI_FORMAT_D32_FLOAT, 
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 }, 
            Usage: D3D11_USAGE_DEFAULT, 
            BindFlags: D3D11_BIND_DEPTH_STENCIL, 
            CPUAccessFlags: D3D11_CPU_ACCESS_FLAG::default(), 
            MiscFlags: D3D11_RESOURCE_MISC_FLAG::default()
        };

        unsafe { 
            self.device.CreateTexture2D(&desc_depth, None, Some(&mut depth_stencil))
            .unwrap_or_else(|e| {
                errors::graphics::GraphicsError::new(
                    &e.message().to_string(),
                    Some(e.code().0),
                    loc!(),
                    Some(self),
                )
            });
        };

        let mut depth_stencil_view: Option<ID3D11DepthStencilView> = None; 
        let depth_stencil_view_desc: D3D11_DEPTH_STENCIL_VIEW_DESC = D3D11_DEPTH_STENCIL_VIEW_DESC { 
            Format: DXGI_FORMAT_D32_FLOAT, 
            ViewDimension: D3D11_DSV_DIMENSION_TEXTURE2D, 
            Flags: 0, 
            Anonymous: D3D11_DEPTH_STENCIL_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_DSV { MipSlice: 0 }
            }
        };

        unsafe { 
            self.device.CreateDepthStencilView(&depth_stencil.unwrap(), Some(&depth_stencil_view_desc), Some(&mut depth_stencil_view))
            .unwrap_or_else(|e| {
                errors::graphics::GraphicsError::new(
                    &e.message().to_string(),
                    Some(e.code().0),
                    loc!(),
                    Some(self),
                )
            });
        };

        self.resources = Some(Resources {
            swap_chain,
            context,
            target: target.unwrap(),
            depth_stencil_view: depth_stencil_view.unwrap()
        });
    }

    fn create_device() -> (IDXGIFactory4, ID3D11Device) {
        let dxgi_factory: IDXGIFactory4 = unsafe { CreateDXGIFactory2(0).unwrap() };

        let mut device: Option<ID3D11Device> = None;
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_DEBUG,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                None,
            )
        }
        .unwrap_or_else(|e| {
            errors::graphics::GraphicsError::new(
                &e.message().to_string(),
                Some(e.code().0),
                loc!(),
                None,
            )
        });

        return (dxgi_factory, device.unwrap());
    }
}

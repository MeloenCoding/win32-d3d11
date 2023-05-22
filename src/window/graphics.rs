use std::slice::from_raw_parts;

use windows::{
    self,
    s,
    Win32::{
        Foundation::S_OK,
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
                D3D11_USAGE_DEFAULT, D3D11_VIEWPORT,
            },
            Dxgi::{
                Common::{
                    DXGI_ALPHA_MODE_UNSPECIFIED, DXGI_FORMAT_R32G32_FLOAT,
                    DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC,
                },
                CreateDXGIFactory2, IDXGIFactory4, IDXGISwapChain1, DXGI_ERROR_DEVICE_REMOVED, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
    },
};

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
}

pub struct VECTOR2 {
    x: f32,
    y: f32,
}

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

    pub fn clear_buffer(&self, rgba: [f32; 4]) {
        unsafe {
            self.resources
                .as_ref()
                .unwrap()
                .context
                .ClearRenderTargetView(&self.resources.as_ref().unwrap().target, &rgba[0])
        };
    }

    pub fn test_triangle(&self) {
        let context = &self.resources.as_ref().unwrap().context;

        // Please ignore :) i hate it as much as you do... and im lazy
        let shader_path = std::env::current_exe().ok().unwrap().parent().unwrap().parent().unwrap().parent().unwrap().join("src\\window\\graphics\\shaders"); 

        // Create triangle vertex's
        // let vertices: Vec<D3DVECTOR> = vec![
        //     D3DVECTOR {x: 0.0, y: 0.5, z: 0.0},
        //     D3DVECTOR {x: 0.5, y: -0.5, z: 0.0},
        //     D3DVECTOR {x: -0.5, y: -0.5, z: 0.0},
        // ];

        let vertices: Vec<VECTOR2> = vec![
            VECTOR2 { x: 0.0, y: 0.5 },
            VECTOR2 { x: 0.5, y: -0.5 },
            VECTOR2 { x: -0.5, y: -0.5 },
        ];

        let compile_flags = D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION;
        // if cfg!(debug_assertions) {
        //     D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
        //     // 0
        // } else {
        //     0
        // };

        // Create a vertex buffer
        let vertex_buff: *mut Option<ID3D11Buffer> = &mut None;

        let buff_desc: D3D11_BUFFER_DESC = D3D11_BUFFER_DESC {
            ByteWidth: std::mem::size_of_val(&vertices) as u32,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_VERTEX_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_FLAG::default(),
            MiscFlags: D3D11_RESOURCE_MISC_FLAG::default(),
            StructureByteStride: std::mem::size_of::<VECTOR2>() as u32,
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

        // Create VertexBuffer on the Input Assembler (IA) [see](https://learn.microsoft.com/en-us/windows/win32/direct3d11/overviews-direct3d-11-graphics-pipeline)
        let mut stride = std::mem::size_of::<VECTOR2>() as u32;
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
        unsafe { context.OMSetRenderTargets(pprendertargetviews, None) };

        // Configure viewport
        let view_port: D3D11_VIEWPORT = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: self.window_width as f32,
            Height: self.window_height as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };

        unsafe {
            context.RSSetViewports(Some(
                Some(view_port).as_ref().map(core::slice::from_ref).unwrap(),
            ))
        };

        unsafe { context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST) }

        let element_desc = D3D11_INPUT_ELEMENT_DESC {
            SemanticName: s!("Position"),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 0,
            InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        };

        let input_layout: Option<*mut Option<ID3D11InputLayout>> = Some(&mut None);

        unsafe {
            self.device.CreateInputLayout(
                Some(element_desc)
                    .as_ref()
                    .map(core::slice::from_ref)
                    .unwrap(),
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

        unsafe { context.Draw(vertices.len().try_into().unwrap(), 0) };
    }

    fn bind_to_window(&mut self, hwnd: &windows::Win32::Foundation::HWND) {
        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: 0,
            Height: 0,
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

        self.resources = Some(Resources {
            swap_chain,
            context,
            target: target.unwrap(),
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

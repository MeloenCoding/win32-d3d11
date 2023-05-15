use windows::{
    self,
    Win32::{
        Foundation::S_OK,
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
                ID3D11Resource, D3D11_SDK_VERSION,
            },
            Dxgi::{
                Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC},
                CreateDXGIFactory2, IDXGIFactory4, IDXGISwapChain1, DXGI_ERROR_DEVICE_REMOVED,
                DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
    },
};

use crate::loc;

use super::errors::{self, FatalErrorBase};

pub struct Graphics {
    dxgi_factory: IDXGIFactory4,
    device: ID3D11Device,
    resources: Option<Resources>,
}

struct Resources {
    pub swap_chain: IDXGISwapChain1,
    pub context: ID3D11DeviceContext,
    pub target: ID3D11RenderTargetView,
}

impl Graphics {
    pub fn setup(hwnd: windows::Win32::Foundation::HWND) -> Graphics {
        let (dxgi_factory, device) = Graphics::create_device();
        let mut graphics = Graphics {
            dxgi_factory,
            device,
            resources: None,
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

    fn bind_to_window(&mut self, hwnd: &windows::Win32::Foundation::HWND) {
        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            BufferCount: 1,
            Width: 0,
            Height: 0,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let swap_chain: IDXGISwapChain1 = unsafe {
            self.dxgi_factory
                .CreateSwapChainForHwnd(&self.device, *hwnd, &swap_chain_desc, None, None)
                .unwrap()
        };

        let context = unsafe { self.device.GetImmediateContext() }.unwrap();
        let back_buffer: ID3D11Resource = unsafe { swap_chain.GetBuffer(0).unwrap() };
        let mut target: Option<ID3D11RenderTargetView> = None;
        unsafe {
            self.device
                .CreateRenderTargetView(&back_buffer, None, Some(&mut target))
                .unwrap()
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
                windows::Win32::Graphics::Direct3D11::D3D11_CREATE_DEVICE_FLAG(0),
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                None,
            )
        }
        .unwrap();

        return (dxgi_factory, device.unwrap());
    }
}

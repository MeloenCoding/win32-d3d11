use windows::{
    self,
    Win32::{Graphics::{
        Direct3D::D3D_DRIVER_TYPE_HARDWARE,
        Direct3D11::{
            D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext, D3D11_SDK_VERSION, D3D11CreateDevice,
        },
        Dxgi::{
            Common::{
                DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_MODE_DESC,
                DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL,
                DXGI_SAMPLE_DESC, DXGI_FORMAT_R8G8B8A8_UNORM,
            },
            IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD,
            DXGI_USAGE_RENDER_TARGET_OUTPUT, CreateDXGIFactory, IDXGIDevice, IDXGIFactory2, CreateDXGIFactory2, DXGI_SWAP_CHAIN_DESC1, IDXGISwapChain1, IDXGIFactory, IDXGIFactory4, IDXGISwapChain3,
        },
    }, Foundation::WPARAM, UI::WindowsAndMessaging::WM_PAINT}, core::ComInterface,
};

pub trait DXGrapics {
    fn setup(hwnd: windows::Win32::Foundation::HWND) -> Self
        where Self: Sized;

    fn end_frame(&self);
    fn bind_to_window(&mut self, hwnd: &windows::Win32::Foundation::HWND);
    fn create_device() -> (IDXGIFactory4, ID3D11Device) where Self: Sized;
}

pub struct Graphics {
    dxgi_factory: IDXGIFactory4,
    device: ID3D11Device,
    resources: Option<Resources>,
}

struct Resources {
    pub swap_chain: IDXGISwapChain3,
}

impl DXGrapics for Graphics {
    fn setup(hwnd: windows::Win32::Foundation::HWND) -> Graphics {
        let (dxgi_factory, device) = Graphics::create_device();
        let mut graphics = Graphics { dxgi_factory, device, resources: None };

        graphics.bind_to_window(&hwnd);

        return graphics;
    }

    fn end_frame(&self) {
        let _ = unsafe { self.resources.as_ref().unwrap().swap_chain.Present(1, 0) };
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

        let swap_chain: IDXGISwapChain3 = unsafe {
            self.dxgi_factory.CreateSwapChainForHwnd(
                &self.device,
                *hwnd,
                &swap_chain_desc,
                None,
                None,
            ).unwrap()
        }
        .cast().unwrap();

        self.resources = Some(Resources {swap_chain});
    }

    fn create_device() -> (IDXGIFactory4, ID3D11Device) {
        let dxgi_factory: IDXGIFactory4 = unsafe {
            CreateDXGIFactory2(0).unwrap()
        };

        let mut device: Option<ID3D11Device> = None;
        unsafe { D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            windows::Win32::Graphics::Direct3D11::D3D11_CREATE_DEVICE_FLAG(0),
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )}.unwrap();

        return (dxgi_factory, device.unwrap());
    }

}

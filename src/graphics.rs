use windows::{
    self,
    Win32::{Graphics::{
        Direct3D::D3D_DRIVER_TYPE_HARDWARE,
        Direct3D11::{
            ID3D11Device, D3D11_SDK_VERSION, D3D11CreateDevice, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Resource,
        },
        Dxgi::{
            Common::{
                DXGI_SAMPLE_DESC, DXGI_FORMAT_R8G8B8A8_UNORM,
            }, DXGI_SWAP_EFFECT_DISCARD,
            DXGI_USAGE_RENDER_TARGET_OUTPUT, CreateDXGIFactory2, DXGI_SWAP_CHAIN_DESC1, IDXGISwapChain1, IDXGIFactory4,
        },
    }},
};


pub struct Graphics {
    dxgi_factory: IDXGIFactory4,
    device: ID3D11Device,
    resources: Option<Resources>,
}

struct Resources {
    pub swap_chain: IDXGISwapChain1,
    pub context: ID3D11DeviceContext,
    pub target: ID3D11RenderTargetView
}

impl Graphics {
    pub fn setup(hwnd: windows::Win32::Foundation::HWND) -> Graphics {
        let (dxgi_factory, device) = Graphics::create_device();
        let mut graphics = Graphics { dxgi_factory, device, resources: None };

        graphics.bind_to_window(&hwnd);

        return graphics;
    }

    pub fn end_frame(&self) {
        let _ = unsafe { self.resources.as_ref().unwrap().swap_chain.Present(1, 0) };
    }

    pub fn clear_buffer(&self, red: i8, green: i8, blue: i8, alpha: i8) {
        
        let rgb: f32 = Self::rgba_to_f32(red, green, blue, alpha);
        unsafe { self.resources.as_ref().unwrap().context.ClearRenderTargetView(&self.resources.as_ref().unwrap().target, &rgb) }
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
            self.dxgi_factory.CreateSwapChainForHwnd(
                &self.device,
                *hwnd,
                &swap_chain_desc,
                None,
                None,
            ).unwrap()
        };

        let context = unsafe { self.device.GetImmediateContext() }.unwrap();
        let back_buffer: ID3D11Resource = unsafe { swap_chain.GetBuffer(0).unwrap() };
        let mut target: Option<ID3D11RenderTargetView> = None;
        unsafe { self.device.CreateRenderTargetView(&back_buffer, None,Some(&mut target)).unwrap() };

        self.resources = Some(Resources {
            swap_chain,
            context,
            target: target.unwrap()
        });
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

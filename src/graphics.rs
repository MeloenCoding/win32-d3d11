use windows::{
    self,
    Win32::Graphics::{
        Direct3D::D3D_DRIVER_TYPE_HARDWARE,
        Direct3D11::{
            D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext, D3D11_SDK_VERSION,
        },
        Dxgi::{
            Common::{
                DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_MODE_DESC,
                DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL,
                DXGI_SAMPLE_DESC,
            },
            IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD,
            DXGI_USAGE_RENDER_TARGET_OUTPUT,
        },
    },
};
pub struct Graphics<'a> {
    pub device: &'a *mut Option<ID3D11Device>,
    pub swap: &'a *mut Option<IDXGISwapChain>,
    pub context: &'a *mut Option<ID3D11DeviceContext>,
}

impl Graphics<'_> {
    pub fn setup(hwnd: windows::Win32::Foundation::HWND) -> Graphics<'static> {
        // https://learn.microsoft.com/en-us/previous-versions/windows/desktop/legacy/bb173064(v=vs.85)
        let swap_desc: *const DXGI_SWAP_CHAIN_DESC = &DXGI_SWAP_CHAIN_DESC {
            BufferDesc: DXGI_MODE_DESC {
                Width: 0,
                Height: 0,
                RefreshRate: DXGI_RATIONAL {
                    Numerator: 0,
                    Denominator: 0,
                },
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
            },
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 1,
            OutputWindow: hwnd,
            Windowed: windows::Win32::Foundation::BOOL(1),
            SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
            Flags: 0,
        };

        let graphics: Graphics = Graphics {
            swap: &(std::ptr::null_mut() as *mut Option<IDXGISwapChain>),
            device: &(std::ptr::null_mut() as *mut Option<ID3D11Device>),
            context: &(std::ptr::null_mut() as *mut Option<ID3D11DeviceContext>),
        };

        let _result: Result<(), windows::core::Error> = unsafe {
            D3D11CreateDeviceAndSwapChain(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                windows::Win32::Graphics::Direct3D11::D3D11_CREATE_DEVICE_FLAG(0),
                None,
                D3D11_SDK_VERSION,
                Some(swap_desc),
                Some(*graphics.swap),
                Some(*graphics.device),
                None,
                Some(*graphics.context),
            )
        };

        return graphics;
    }

    pub fn end_frame(&mut self) {
        unsafe {
            println!("{:?}", self.swap);
        }
    }
}

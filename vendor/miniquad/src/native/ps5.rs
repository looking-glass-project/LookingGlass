//! PS5 platform for miniquad (target `x86_64-sie-ps5-std`, FreeBSD-based).
//!
//! - Console services (klog, splash, DualSense, audio out, exit) come from PS5_VulkanTemplate's
//!   `platform.c` (MIT, Mihawk), copied unchanged into `ps5c/` and compiled by build.rs.
//! - Rendering is the Vulkan backend (`graphics::vulkan`) on RADV, which is linked into the
//!   title statically: the entry point is `vk_icdGetInstanceProcAddr`, as in the template.
//! - Presentation: VK_KHR_display on VideoOut. The game renders at `RENDER_SIZE` into the
//!   backend's default image (GL row order); each frame is blitted, flipped and scaled, to
//!   the display's swapchain image.
//! - Input: `pump` polls the pads every frame; the engine reads them through [`pad`].
use crate::{
    conf::Conf,
    graphics::vulkan::{self, Core, Presenter},
    EventHandler,
};
use ash::{khr, vk};
use std::ffi::{c_char, c_void, CStr};
use std::sync::Mutex;

/// The resolution the game renders at; the display scales it to its own mode (3840x2160).
pub const RENDER_SIZE: (u32, u32) = (1920, 1080);

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CPad {
    held: u32,
    pressed: u32,
    left_x: f32,
    left_y: f32,
    right_x: f32,
    right_y: f32,
    l2: f32,
    r2: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CPadReading {
    buttons: u32,
    left_x: u8,
    left_y: u8,
    right_x: u8,
    right_y: u8,
    l2: u8,
    r2: u8,
    connected: bool,
    timestamp_us: u64,
}

extern "C" {
    fn platform_init(title_name: *const c_char);
    fn pad_open() -> bool;
    fn pad_poll(pad: *mut CPad);
    fn pad_player(player: i32, pad: *mut CPad) -> bool;
    fn pad_player_readings(player: i32, readings: *mut *const CPadReading) -> i32;
}

extern "system" {
    /// RADV's ICD entry point, from the statically linked `libvulkan_radeon.ps5.a`.
    fn vk_icdGetInstanceProcAddr(instance: vk::Instance, name: *const c_char) -> vk::PFN_vkVoidFunction;
}

/// DualSense buttons in the console's numbering (platform.h).
pub mod button {
    pub const L3: u32 = 0x000002;
    pub const R3: u32 = 0x000004;
    pub const OPTIONS: u32 = 0x000008;
    pub const UP: u32 = 0x000010;
    pub const RIGHT: u32 = 0x000020;
    pub const DOWN: u32 = 0x000040;
    pub const LEFT: u32 = 0x000080;
    pub const L1: u32 = 0x000400;
    pub const R1: u32 = 0x000800;
    pub const TRIANGLE: u32 = 0x001000;
    pub const CIRCLE: u32 = 0x002000;
    pub const CROSS: u32 = 0x004000;
    pub const SQUARE: u32 = 0x008000;
    pub const TOUCH_PAD: u32 = 0x100000;
    /// Set while the system (home screen, a dialog) has the pad: not input for the game.
    pub const INTERCEPTED: u32 = 0x80000000;
}

/// One controller's state as of the last frame; sticks 0..255 (128 at rest, y down-positive).
#[derive(Default, Clone, Copy, Debug)]
pub struct PadState {
    pub buttons: u32,
    pub left_x: u8,
    pub left_y: u8,
    pub right_x: u8,
    pub right_y: u8,
    pub l2: u8,
    pub r2: u8,
}

static PADS: Mutex<[Option<PadState>; 4]> = Mutex::new([None; 4]);

/// Player `player`'s controller (0..4), or `None` when that player has none.
pub fn pad(player: usize) -> Option<PadState> {
    PADS.lock().unwrap().get(player).copied().flatten()
}

unsafe fn poll_pads() {
    let mut p0 = CPad::default();
    pad_poll(&mut p0);
    let mut pads = [None; 4];
    for (player, slot) in pads.iter_mut().enumerate() {
        let mut p = CPad::default();
        if !pad_player(player as i32, &mut p) { continue; }
        let mut readings: *const CPadReading = std::ptr::null();
        let n = pad_player_readings(player as i32, &mut readings);
        if n <= 0 || readings.is_null() { continue; }
        let r = *readings.add(n as usize - 1);
        if !r.connected || r.buttons & button::INTERCEPTED != 0 { continue; }
        *slot = Some(PadState { buttons: r.buttons, left_x: r.left_x, left_y: r.left_y, right_x: r.right_x, right_y: r.right_y, l2: r.l2, r2: r.r2 });
    }
    *PADS.lock().unwrap() = pads;
}

/// VK_KHR_display presentation.
struct DisplayPresenter {
    render: (u32, u32),
    surface_fn: Option<khr::surface::Instance>,
    swapchain_fn: Option<khr::swapchain::Device>,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    extent: vk::Extent2D,
    acquired: vk::Semaphore,
    rendered: vk::Semaphore,
    index: Option<u32>,
}

impl Presenter for DisplayPresenter {
    fn instance_extensions(&self) -> Vec<&'static CStr> { vec![khr::surface::NAME, khr::display::NAME] }
    fn device_extensions(&self) -> Vec<&'static CStr> { vec![khr::swapchain::NAME] }
    fn entry(&self) -> Option<ash::Entry> {
        Some(unsafe { ash::Entry::from_static_fn(ash::StaticFn { get_instance_proc_addr: vk_icdGetInstanceProcAddr }) })
    }
    fn size(&self) -> (u32, u32) { self.render }

    fn init(&mut self, core: &Core) {
        unsafe { self.create(core) }.unwrap_or_else(|e| panic!("{}", format!("display setup failed: {e:?}")));
    }

    fn present_record(&mut self, core: &Core, cmd: vk::CommandBuffer, image: vk::Image, w: u32, h: u32) -> (Vec<vk::Semaphore>, Vec<vk::Semaphore>) {
        let sc = self.swapchain_fn.as_ref().unwrap();
        let (index, _) = match unsafe { sc.acquire_next_image(self.swapchain, u64::MAX, self.acquired, vk::Fence::null()) } {
            Ok(r) => r,
            Err(e) => { eprintln!("ps5: acquire failed: {e:?}"); self.index = None; return (vec![], vec![]); }
        };
        self.index = Some(index);
        let dst = self.images[index as usize];
        let d = &core.device;
        let range = vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 };
        let barrier = |from, to, src, dstm| vk::ImageMemoryBarrier::default()
            .src_access_mask(src).dst_access_mask(dstm).old_layout(from).new_layout(to)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED).dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(dst).subresource_range(range);
        unsafe {
            d.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::TOP_OF_PIPE, vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(), &[], &[],
                &[barrier(vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::AccessFlags::empty(), vk::AccessFlags::TRANSFER_WRITE)]);
            let sub = vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 };
            // Source rows are bottom-up (GL order): flip while scaling to the display.
            let blit = vk::ImageBlit {
                src_subresource: sub,
                src_offsets: [vk::Offset3D { x: 0, y: h as i32, z: 0 }, vk::Offset3D { x: w as i32, y: 0, z: 1 }],
                dst_subresource: sub,
                dst_offsets: [vk::Offset3D::default(), vk::Offset3D { x: self.extent.width as i32, y: self.extent.height as i32, z: 1 }],
            };
            d.cmd_blit_image(cmd, image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, dst, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[blit], vk::Filter::LINEAR);
            d.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::TRANSFER, vk::PipelineStageFlags::BOTTOM_OF_PIPE, vk::DependencyFlags::empty(), &[], &[],
                &[barrier(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::PRESENT_SRC_KHR, vk::AccessFlags::TRANSFER_WRITE, vk::AccessFlags::empty())]);
        }
        (vec![self.acquired], vec![self.rendered])
    }

    fn present_finish(&mut self, core: &Core) {
        let Some(index) = self.index.take() else { return };
        let sc = self.swapchain_fn.as_ref().unwrap();
        let (s, w, i) = ([self.swapchain], [self.rendered], [index]);
        if let Err(e) = unsafe { sc.queue_present(core.queue, &vk::PresentInfoKHR::default().swapchains(&s).wait_semaphores(&w).image_indices(&i)) } {
            eprintln!("ps5: present failed: {e:?}");
        }
    }
}

impl DisplayPresenter {
    unsafe fn create(&mut self, core: &Core) -> Result<(), vk::Result> {
        let disp = khr::display::Instance::new(&core.entry, &core.instance);
        let pdev = core.pdev;
        let displays = disp.get_physical_device_display_properties(pdev)?;
        let display = displays.first().ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?.display;
        // The console's mode: the one with the highest refresh at the largest size.
        let modes = disp.get_display_mode_properties(pdev, display)?;
        let mode = modes
            .iter()
            .max_by_key(|m| (m.parameters.visible_region.width * m.parameters.visible_region.height, m.parameters.refresh_rate))
            .ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?;
        let planes = disp.get_physical_device_display_plane_properties(pdev)?;
        let mut plane = None;
        for i in 0..planes.len() as u32 {
            if disp.get_display_plane_supported_displays(pdev, i)?.contains(&display) { plane = Some(i); break; }
        }
        let plane = plane.ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?;
        let caps = disp.get_display_plane_capabilities(pdev, mode.display_mode, plane)?;
        let alpha = [vk::DisplayPlaneAlphaFlagsKHR::OPAQUE, vk::DisplayPlaneAlphaFlagsKHR::GLOBAL, vk::DisplayPlaneAlphaFlagsKHR::PER_PIXEL, vk::DisplayPlaneAlphaFlagsKHR::PER_PIXEL_PREMULTIPLIED]
            .iter().copied().find(|a| caps.supported_alpha.contains(*a)).unwrap_or(vk::DisplayPlaneAlphaFlagsKHR::OPAQUE);
        let extent = mode.parameters.visible_region;
        let surface = disp.create_display_plane_surface(
            &vk::DisplaySurfaceCreateInfoKHR::default()
                .display_mode(mode.display_mode)
                .plane_index(plane)
                .plane_stack_index(planes[plane as usize].current_stack_index)
                .transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
                .global_alpha(1.0)
                .alpha_mode(alpha)
                .image_extent(extent),
            None,
        )?;
        let surface_fn = khr::surface::Instance::new(&core.entry, &core.instance);
        let scaps = surface_fn.get_physical_device_surface_capabilities(pdev, surface)?;
        let formats = surface_fn.get_physical_device_surface_formats(pdev, surface)?;
        let format = formats
            .iter()
            .find(|f| matches!(f.format, vk::Format::B8G8R8A8_UNORM | vk::Format::R8G8B8A8_UNORM))
            .or(formats.first())
            .copied()
            .ok_or(vk::Result::ERROR_FORMAT_NOT_SUPPORTED)?;
        let count = (scaps.min_image_count + 1).min(if scaps.max_image_count == 0 { u32::MAX } else { scaps.max_image_count });
        let swapchain_fn = khr::swapchain::Device::new(&core.instance, &core.device);
        let swapchain = swapchain_fn.create_swapchain(
            &vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(count)
                .image_format(format.format)
                .image_color_space(format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::FIFO)
                .clipped(true),
            None,
        )?;
        self.images = swapchain_fn.get_swapchain_images(swapchain)?;
        self.acquired = core.device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;
        self.rendered = core.device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;
        eprintln!("ps5: display {}x{} @ {} mHz, {} swapchain images {:?}, rendering at {}x{}",
            extent.width, extent.height, mode.parameters.refresh_rate, self.images.len(), format.format, self.render.0, self.render.1);
        self.extent = extent;
        self.swapchain = swapchain;
        self.surface_fn = Some(surface_fn);
        self.swapchain_fn = Some(swapchain_fn);
        Ok(())
    }
}

pub fn run<F>(conf: &Conf, f: F)
where
    F: 'static + FnOnce() -> Box<dyn EventHandler>,
{
    unsafe {
        platform_init(b"Looking Glass\0".as_ptr() as *const c_char);
        if !pad_open() { eprintln!("ps5: no controller yet"); }
    }
    vulkan::set_presenter(Box::new(DisplayPresenter {
        render: RENDER_SIZE,
        surface_fn: None,
        swapchain_fn: None,
        swapchain: vk::SwapchainKHR::null(),
        images: vec![],
        extent: vk::Extent2D::default(),
        acquired: vk::Semaphore::null(),
        rendered: vk::Semaphore::null(),
        index: None,
    }));
    let size = (RENDER_SIZE.0 as i32, RENDER_SIZE.1 as i32);
    crate::native::vulkan_headless::run_with(conf, size, f, |_handler: &mut dyn EventHandler| unsafe { poll_pads() });
    let _ = std::ptr::null::<c_void>();
}

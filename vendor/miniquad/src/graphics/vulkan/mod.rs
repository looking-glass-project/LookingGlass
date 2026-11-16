//! Vulkan implementation of miniquad's `RenderingBackend`, written for the PS5 port of
//! Looking Glass (RADV on the console, lavapipe for tests on Linux).
//!
//! Design (see also glsl.rs):
//! - Shaders arrive as GLSL ES 1.00 (we report `Backend::OpenGl` with only v100 support) and
//!   are translated to SPIR-V with naga at `new_shader`.
//! - Every target is rendered in GL's memory layout (row 0 = bottom). Render-to-texture,
//!   gl_FragCoord, scissor rects and read-backs therefore match GL; the platform flips when
//!   it presents. Because Y is not flipped, GL's CCW front faces are Vulkan's CW.
//! - One frame in flight; a host-visible ring per frame carries uniforms, stream/dynamic
//!   buffer contents and upload staging. When the ring fills, the frame is flushed
//!   (submit + wait) and recording continues, restarting the render pass with LOAD.
//! - Descriptors are pushed per draw (VK_KHR_push_descriptor).
//! - Transfers that GL would order inside a frame (texture updates of textures already
//!   drawn this frame, read-backs, screen grabs, mipmap generation) split the render pass.

mod alloc;
pub mod glsl;

use crate::graphics::*;
use crate::ResourceManager;
use alloc::{Allocation, Allocator};
use ash::{khr, vk};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::Mutex;

const RING_SIZE: u64 = 96 << 20;

/// Hooks the platform provides: presentation and the extensions it needs. Install with
/// [`set_presenter`] before the backend is created.
pub trait Presenter: Send {
    fn instance_extensions(&self) -> Vec<&'static CStr> { vec![] }
    fn device_extensions(&self) -> Vec<&'static CStr> { vec![] }
    /// The Vulkan entry. `None` loads the system's libvulkan; the PS5 links RADV statically
    /// and hands its `vk_icdGetInstanceProcAddr` over here.
    fn entry(&self) -> Option<ash::Entry> { None }
    /// Called once after the device exists.
    fn init(&mut self, _core: &Core) {}
    /// Size of the default framebuffer.
    fn size(&self) -> (u32, u32);
    /// Record the hand-off of `image` (layout TRANSFER_SRC_OPTIMAL, GL row order) into `cmd`.
    /// Returns semaphores the submit must wait on and signal.
    fn present_record(&mut self, _core: &Core, _cmd: vk::CommandBuffer, _image: vk::Image, _w: u32, _h: u32)
        -> (Vec<vk::Semaphore>, Vec<vk::Semaphore>) { (vec![], vec![]) }
    /// Called after the frame's submission.
    fn present_finish(&mut self, _core: &Core) {}
}

struct Headless(u32, u32);
impl Presenter for Headless {
    fn size(&self) -> (u32, u32) { (self.0, self.1) }
}

static PRESENTER: Mutex<Option<Box<dyn Presenter>>> = Mutex::new(None);
pub fn set_presenter(p: Box<dyn Presenter>) { *PRESENTER.lock().unwrap() = Some(p); }

pub struct Core {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub pdev: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queue: vk::Queue,
    pub queue_family: u32,
}

struct Shader {
    vs: vk::ShaderModule,
    fs: vk::ShaderModule,
    set_layout: vk::DescriptorSetLayout,
    layout: vk::PipelineLayout,
    uniforms: Vec<UniformDesc>,
    std140: Vec<(usize, usize, usize, usize)>, // (src offset, dst offset, element bytes, dst stride) per element
    ubo_size: usize,
    images: usize,
    attributes: HashMap<String, u32>,
}

struct PipelineData {
    shader: ShaderId,
    layouts: Vec<BufferLayout>,
    attributes: Vec<VertexAttribute>,
    params: PipelineParams,
    built: HashMap<RpKey, vk::Pipeline>,
}

struct Buffer {
    buf: vk::Buffer,
    mem: Allocation,
    size: usize,
    index_type: vk::IndexType,
    dynamic: bool,
    current: (vk::Buffer, u64, *mut u8), // where the latest contents live
    in_ring_len: usize,
}

struct Texture {
    image: vk::Image,
    view: vk::ImageView,
    mem: Allocation,
    format: vk::Format,
    params: TextureParams,
    levels: u32,
    depth: bool,
    used_frame: u64,
}

struct Pass {
    colors: Vec<TextureId>,
    depth: Option<TextureId>,
    fb: vk::Framebuffer,
    key: RpKey,
    w: u32,
    h: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct RpKey {
    colors: [vk::Format; 4],
    ncolors: u8,
    depth: vk::Format,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct RpVariant {
    key: RpKey,
    clear_color: bool,
    clear_depth: bool,
    clear_stencil: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SamplerKey(u8, u8, u8, u8, u32);

#[derive(Clone, Copy)]
enum Target { Default, Pass(usize) }

struct Active {
    target: Target,
    rp_key: RpKey,
    fb: vk::Framebuffer,
    w: u32,
    h: u32,
}

struct Ring {
    buf: vk::Buffer,
    mem: Allocation,
    off: u64,
}

struct Inner {
    core: Core,
    push: khr::push_descriptor::Device,
    alloc: Allocator,
    ubo_align: u64,
    cmd_pool: vk::CommandPool,
    cmd: vk::CommandBuffer,
    upload: vk::CommandBuffer,
    upload_open: bool,
    fence: vk::Fence,
    ring: Ring,
    frame: u64,
    shaders: ResourceManager<Shader>,
    pipelines: ResourceManager<PipelineData>,
    buffers: ResourceManager<Buffer>,
    textures: Vec<Option<Texture>>,
    passes: ResourceManager<Pass>,
    render_passes: HashMap<RpVariant, vk::RenderPass>,
    samplers: HashMap<SamplerKey, vk::Sampler>,
    default_color: TextureId,
    default_depth: TextureId,
    default_fb: vk::Framebuffer,
    default_key: RpKey,
    depth_format: vk::Format,
    default_depth_format: vk::Format,
    active: Option<Active>,
    pipeline: Option<Pipeline>,
    vbufs: Vec<BufferId>,
    ibuf: Option<BufferId>,
    images: Vec<TextureId>,
    uniforms: Vec<u8>,
    viewport: (i32, i32, i32, i32),
    scissor: (i32, i32, i32, i32),
    depth_write_override: Option<bool>,
    dynamic_in_ring: Vec<usize>,
    trash: Vec<Box<dyn FnOnce(&ash::Device, &mut Allocator)>>,
    presenter: Box<dyn Presenter>,
    info: ContextInfo,
}

pub struct VulkanContext(RefCell<Inner>);

fn vk_check<T>(r: Result<T, vk::Result>, what: &str) -> T {
    r.unwrap_or_else(|e| panic!("{}", format!("{what}: {e:?}")))
}

impl VulkanContext {
    pub fn new() -> VulkanContext {
        unsafe { VulkanContext(RefCell::new(Inner::new())) }
    }
}

impl Inner {
    unsafe fn new() -> Inner {
        let presenter = PRESENTER.lock().unwrap().take().unwrap_or_else(|| {
            let (w, h) = crate::window::screen_size();
            Box::new(Headless(w as u32, h as u32))
        });
        let entry = match presenter.entry() {
            Some(e) => e,
            None => vk_check(ash::Entry::load().map_err(|_| vk::Result::ERROR_INITIALIZATION_FAILED), "load Vulkan"),
        };
        let app = vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 1, 0));
        let iext: Vec<*const i8> = presenter.instance_extensions().iter().map(|e| e.as_ptr()).collect();
        let instance = vk_check(
            entry.create_instance(&vk::InstanceCreateInfo::default().application_info(&app).enabled_extension_names(&iext), None),
            "vkCreateInstance",
        );
        let pdevs = vk_check(instance.enumerate_physical_devices(), "enumerate devices");
        let pdev = *pdevs.first().expect("no Vulkan device");
        let props = instance.get_physical_device_properties(pdev);
        let queue_family = instance
            .get_physical_device_queue_family_properties(pdev)
            .iter()
            .position(|q| q.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .expect("no graphics queue") as u32;
        let prio = [1.0f32];
        let qci = [vk::DeviceQueueCreateInfo::default().queue_family_index(queue_family).queue_priorities(&prio)];
        let mut dext: Vec<*const i8> = vec![khr::push_descriptor::NAME.as_ptr()];
        dext.extend(presenter.device_extensions().iter().map(|e| e.as_ptr()));
        let device = vk_check(
            instance.create_device(pdev, &vk::DeviceCreateInfo::default().queue_create_infos(&qci).enabled_extension_names(&dext), None),
            "vkCreateDevice",
        );
        let queue = device.get_device_queue(queue_family, 0);
        let core = Core { entry, instance, pdev, device, queue, queue_family };
        let push = khr::push_descriptor::Device::new(&core.instance, &core.device);
        let alloc = Allocator::new(core.instance.get_physical_device_memory_properties(pdev));
        let cmd_pool = vk_check(
            core.device.create_command_pool(
                &vk::CommandPoolCreateInfo::default().queue_family_index(queue_family).flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            ),
            "command pool",
        );
        let cbs = vk_check(
            core.device.allocate_command_buffers(&vk::CommandBufferAllocateInfo::default().command_pool(cmd_pool).command_buffer_count(2)),
            "command buffers",
        );
        let fence = vk_check(core.device.create_fence(&vk::FenceCreateInfo::default(), None), "fence");

        let pick = |cands: &[vk::Format]| {
            *cands
                .iter()
                .find(|f| {
                    core.instance
                        .get_physical_device_format_properties(pdev, **f)
                        .optimal_tiling_features
                        .contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
                })
                .expect("no depth format")
        };
        let depth_format = pick(&[vk::Format::D32_SFLOAT, vk::Format::D24_UNORM_S8_UINT]);
        let default_depth_format = pick(&[vk::Format::D24_UNORM_S8_UINT, vk::Format::D32_SFLOAT_S8_UINT]);

        let mut inner = Inner {
            push,
            alloc,
            ubo_align: props.limits.min_uniform_buffer_offset_alignment.max(16),
            cmd_pool,
            cmd: cbs[0],
            upload: cbs[1],
            upload_open: false,
            fence,
            ring: Ring { buf: vk::Buffer::null(), mem: std::mem::zeroed(), off: 0 },
            frame: 1,
            shaders: Default::default(),
            pipelines: Default::default(),
            buffers: Default::default(),
            textures: vec![],
            passes: Default::default(),
            render_passes: HashMap::new(),
            samplers: HashMap::new(),
            default_color: TextureId(TextureIdInner::Managed(0)),
            default_depth: TextureId(TextureIdInner::Managed(0)),
            default_fb: vk::Framebuffer::null(),
            default_key: RpKey { colors: [vk::Format::UNDEFINED; 4], ncolors: 0, depth: vk::Format::UNDEFINED },
            depth_format,
            default_depth_format,
            active: None,
            pipeline: None,
            vbufs: vec![],
            ibuf: None,
            images: vec![],
            uniforms: vec![],
            viewport: (0, 0, 0, 0),
            scissor: (0, 0, 0, 0),
            depth_write_override: None,
            dynamic_in_ring: vec![],
            trash: vec![],
            presenter,
            info: ContextInfo {
                backend: Backend::OpenGl,
                gl_version_string: format!("Vulkan (miniquad PS5 backend) on {}", CStr::from_ptr(props.device_name.as_ptr()).to_string_lossy()),
                glsl_support: GlslSupport { v100: true, ..Default::default() },
                features: Features { instancing: true, resolve_attachments: false },
            },
            core,
        };
        let (rb, rm) = inner.make_buffer(RING_SIZE as usize, vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST);
        inner.ring = Ring { buf: rb, mem: rm, off: 0 };
        inner.presenter.init(&inner.core);
        inner.begin_cmd();
        let (w, h) = inner.presenter.size();
        inner.create_default_target(w, h);
        eprintln!("miniquad: {}", inner.info.gl_version_string);
        inner
    }

    unsafe fn make_buffer(&mut self, size: usize, usage: vk::BufferUsageFlags) -> (vk::Buffer, Allocation) {
        let d = &self.core.device;
        let buf = vk_check(d.create_buffer(&vk::BufferCreateInfo::default().size(size.max(4) as u64).usage(usage), None), "buffer");
        let mem = self.alloc.alloc(d, d.get_buffer_memory_requirements(buf), true);
        vk_check(d.bind_buffer_memory(buf, mem.memory, mem.offset), "bind buffer");
        (buf, mem)
    }

    unsafe fn begin_cmd(&mut self) {
        vk_check(
            self.core.device.begin_command_buffer(self.cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)),
            "begin cmd",
        );
    }

    unsafe fn upload_cmd(&mut self) -> vk::CommandBuffer {
        if !self.upload_open {
            vk_check(
                self.core.device.begin_command_buffer(self.upload, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)),
                "begin upload",
            );
            self.upload_open = true;
        }
        self.upload
    }

    unsafe fn full_barrier(&self, cmd: vk::CommandBuffer) {
        let mb = [vk::MemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE)];
        self.core.device.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::ALL_COMMANDS, vk::PipelineStageFlags::ALL_COMMANDS, vk::DependencyFlags::empty(), &mb, &[], &[]);
    }

    unsafe fn layout_barrier(&self, cmd: vk::CommandBuffer, t: &Texture, from: vk::ImageLayout, to: vk::ImageLayout) {
        let aspect = aspect_of(t.format);
        let ib = [vk::ImageMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE)
            .old_layout(from)
            .new_layout(to)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(t.image)
            .subresource_range(vk::ImageSubresourceRange { aspect_mask: aspect, base_mip_level: 0, level_count: vk::REMAINING_MIP_LEVELS, base_array_layer: 0, layer_count: vk::REMAINING_ARRAY_LAYERS })];
        self.core.device.cmd_pipeline_barrier(cmd, vk::PipelineStageFlags::ALL_COMMANDS, vk::PipelineStageFlags::ALL_COMMANDS, vk::DependencyFlags::empty(), &[], &[], &ib);
    }

    /// Bytes in the ring; flushes the frame when full. Returns (buffer, offset, ptr).
    unsafe fn ring_alloc(&mut self, size: usize, align: u64) -> (vk::Buffer, u64, *mut u8) {
        let size = size as u64;
        if size > RING_SIZE { panic!("{}", format!("single upload of {size} bytes exceeds the {RING_SIZE}-byte ring")) }
        let mut off = (self.ring.off + align - 1) / align * align;
        if off + size > RING_SIZE {
            self.flush();
            off = 0;
        }
        self.ring.off = off + size;
        (self.ring.buf, off, self.ring.mem.ptr.add(off as usize))
    }

    /// Submit what has been recorded, wait, and continue the frame (the render pass, if any,
    /// restarts with LOAD).
    unsafe fn flush(&mut self) {
        let resume = self.active.as_ref().map(|a| a.target);
        if resume.is_some() { self.end_pass_inner(); }
        self.submit(false);
        if let Some(t) = resume { self.begin_target(t, None); }
    }

    unsafe fn submit(&mut self, present: bool) {
        let d = self.core.device.clone();
        let (mut waits, mut signals) = (vec![], vec![]);
        if present {
            let t = self.tex(self.default_color);
            let (image, w, h) = (t.image, t.params.width, t.params.height);
            let tt = self.tex(self.default_color) as *const Texture;
            self.layout_barrier(self.cmd, &*tt, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
            let (wa, si) = self.presenter.present_record(&self.core, self.cmd, image, w, h);
            self.layout_barrier(self.cmd, &*tt, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            waits = wa;
            signals = si;
        }
        vk_check(d.end_command_buffer(self.cmd), "end cmd");
        let mut cbs = vec![];
        if self.upload_open {
            self.full_barrier(self.upload);
            vk_check(d.end_command_buffer(self.upload), "end upload");
            cbs.push(self.upload);
            self.upload_open = false;
        }
        cbs.push(self.cmd);
        let stages = vec![vk::PipelineStageFlags::ALL_COMMANDS; waits.len()];
        let si = [vk::SubmitInfo::default().command_buffers(&cbs).wait_semaphores(&waits).wait_dst_stage_mask(&stages).signal_semaphores(&signals)];
        vk_check(d.queue_submit(self.core.queue, &si, self.fence), "vkQueueSubmit");
        vk_check(d.wait_for_fences(&[self.fence], true, u64::MAX), "wait fence");
        vk_check(d.reset_fences(&[self.fence]), "reset fence");
        if present { self.presenter.present_finish(&self.core); }
        // Preserve the newest contents of dynamic buffers that live in the ring.
        for id in std::mem::take(&mut self.dynamic_in_ring) {
            if let Some(b) = self.buffers.resources.get_mut(&id) {
                if b.current.0 == self.ring.buf {
                    std::ptr::copy_nonoverlapping(b.current.2, b.mem.ptr, b.in_ring_len);
                    b.current = (b.buf, 0, b.mem.ptr);
                }
            }
        }
        self.ring.off = 0;
        for f in std::mem::take(&mut self.trash) { f(&d, &mut self.alloc); }
        self.begin_cmd();
    }

    fn tex(&self, id: TextureId) -> &Texture {
        match id.0 {
            TextureIdInner::Managed(i) => self.textures[i].as_ref().expect("deleted texture"),
            TextureIdInner::Raw(_) => panic!("raw texture ids are not supported by the Vulkan backend"),
        }
    }
    fn tex_mut(&mut self, id: TextureId) -> &mut Texture {
        match id.0 {
            TextureIdInner::Managed(i) => self.textures[i].as_mut().expect("deleted texture"),
            TextureIdInner::Raw(_) => panic!("raw texture ids are not supported by the Vulkan backend"),
        }
    }

    unsafe fn create_default_target(&mut self, w: u32, h: u32) {
        let c = self.new_texture(TextureAccess::RenderTarget, TextureSource::Empty, TextureParams { width: w, height: h, ..Default::default() }, None);
        let dpt = self.new_texture(
            TextureAccess::RenderTarget,
            TextureSource::Empty,
            TextureParams { width: w, height: h, format: TextureFormat::Depth, ..Default::default() },
            Some(self.default_depth_format),
        );
        let key = RpKey { colors: [vk::Format::R8G8B8A8_UNORM, vk::Format::UNDEFINED, vk::Format::UNDEFINED, vk::Format::UNDEFINED], ncolors: 1, depth: self.default_depth_format };
        let views = [self.tex(c).view, self.tex(dpt).view];
        let rp = self.render_pass(RpVariant { key, clear_color: false, clear_depth: false, clear_stencil: false });
        self.default_fb = vk_check(
            self.core.device.create_framebuffer(&vk::FramebufferCreateInfo::default().render_pass(rp).attachments(&views).width(w).height(h).layers(1), None),
            "default framebuffer",
        );
        self.default_color = c;
        self.default_depth = dpt;
        self.default_key = key;
    }

    unsafe fn render_pass(&mut self, v: RpVariant) -> vk::RenderPass {
        if let Some(rp) = self.render_passes.get(&v) { return *rp; }
        let mut atts = vec![];
        let mut refs = vec![];
        for i in 0..v.key.ncolors as usize {
            atts.push(
                vk::AttachmentDescription::default()
                    .format(v.key.colors[i])
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(if v.clear_color { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD })
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL),
            );
            refs.push(vk::AttachmentReference { attachment: i as u32, layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL });
        }
        let dref = vk::AttachmentReference { attachment: v.key.ncolors as u32, layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL };
        let has_depth = v.key.depth != vk::Format::UNDEFINED;
        if has_depth {
            atts.push(
                vk::AttachmentDescription::default()
                    .format(v.key.depth)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(if v.clear_depth { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD })
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(if v.clear_stencil { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD })
                    .stencil_store_op(vk::AttachmentStoreOp::STORE)
                    .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            );
        }
        let mut sub = vk::SubpassDescription::default().pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS).color_attachments(&refs);
        if has_depth { sub = sub.depth_stencil_attachment(&dref); }
        let all = vk::PipelineStageFlags::ALL_COMMANDS;
        let gfx = vk::PipelineStageFlags::ALL_GRAPHICS; // the subpass side may only name graphics stages
        let rw = vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE;
        let deps = [
            vk::SubpassDependency { src_subpass: vk::SUBPASS_EXTERNAL, dst_subpass: 0, src_stage_mask: all, dst_stage_mask: gfx, src_access_mask: vk::AccessFlags::MEMORY_WRITE, dst_access_mask: rw, dependency_flags: vk::DependencyFlags::empty() },
            vk::SubpassDependency { src_subpass: 0, dst_subpass: vk::SUBPASS_EXTERNAL, src_stage_mask: gfx, dst_stage_mask: all, src_access_mask: vk::AccessFlags::MEMORY_WRITE, dst_access_mask: rw, dependency_flags: vk::DependencyFlags::empty() },
        ];
        let subs = [sub];
        let rp = vk_check(
            self.core.device.create_render_pass(&vk::RenderPassCreateInfo::default().attachments(&atts).subpasses(&subs).dependencies(&deps), None),
            "render pass",
        );
        self.render_passes.insert(v, rp);
        rp
    }

    unsafe fn new_texture(&mut self, access: TextureAccess, source: TextureSource, params: TextureParams, force_format: Option<vk::Format>) -> TextureId {
        if params.kind == TextureKind::CubeMap { panic!("cube maps are not supported by the Vulkan backend yet") }
        // GL can glGenerateMipmap any texture later; allocate the chain for static textures so
        // texture_generate_mipmaps works the same (unused levels are never sampled before then,
        // because the sampler clamps to level 0 while the mipmap filter is None).
        let want_mips = params.allocate_mipmaps || (access == TextureAccess::Static && force_format.is_none());
        let format = force_format.unwrap_or(match params.format {
            TextureFormat::RGB8 | TextureFormat::RGBA8 => vk::Format::R8G8B8A8_UNORM,
            TextureFormat::RGBA16F => vk::Format::R16G16B16A16_SFLOAT,
            TextureFormat::Depth | TextureFormat::Depth32 => self.depth_format,
            TextureFormat::Alpha => vk::Format::R8_UNORM,
        });
        let depth = aspect_of(format).contains(vk::ImageAspectFlags::DEPTH);
        let levels = if want_mips { 32 - params.width.max(params.height).max(1).leading_zeros() } else { 1 };
        let usage = vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC
            | if depth { vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT } else { vk::ImageUsageFlags::COLOR_ATTACHMENT };
        let d = self.core.device.clone();
        let image = vk_check(
            d.create_image(
                &vk::ImageCreateInfo::default()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(format)
                    .extent(vk::Extent3D { width: params.width.max(1), height: params.height.max(1), depth: 1 })
                    .mip_levels(levels)
                    .array_layers(1)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .usage(usage)
                    .initial_layout(vk::ImageLayout::UNDEFINED),
                None,
            ),
            "vkCreateImage",
        );
        let mem = self.alloc.alloc(&d, d.get_image_memory_requirements(image), false);
        vk_check(d.bind_image_memory(image, mem.memory, mem.offset), "bind image");
        let swz = if params.format == TextureFormat::Alpha {
            vk::ComponentMapping { r: vk::ComponentSwizzle::IDENTITY, g: vk::ComponentSwizzle::IDENTITY, b: vk::ComponentSwizzle::IDENTITY, a: vk::ComponentSwizzle::R }
        } else if params.format == TextureFormat::RGB8 {
            vk::ComponentMapping { a: vk::ComponentSwizzle::ONE, ..Default::default() }
        } else {
            vk::ComponentMapping::default()
        };
        let view = vk_check(
            d.create_image_view(
                &vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .components(swz)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: if depth { vk::ImageAspectFlags::DEPTH } else { vk::ImageAspectFlags::COLOR },
                        base_mip_level: 0,
                        level_count: levels,
                        base_array_layer: 0,
                        layer_count: 1,
                    }),
                None,
            ),
            "image view",
        );
        let t = Texture { image, view, mem, format, params, levels, depth, used_frame: 0 };
        let rest = resting(&t);
        let cmd = self.upload_cmd();
        self.layout_barrier(cmd, &t, vk::ImageLayout::UNDEFINED, rest);
        if !depth {
            // GL leaves new textures undefined; clear to transparent black for determinism.
            self.layout_barrier(cmd, &t, rest, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
            d.cmd_clear_color_image(cmd, t.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &vk::ClearColorValue::default(), &[vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: levels, base_array_layer: 0, layer_count: 1 }]);
            self.layout_barrier(cmd, &t, vk::ImageLayout::TRANSFER_DST_OPTIMAL, rest);
        }
        let id = match self.textures.iter().position(|t| t.is_none()) {
            Some(i) => { self.textures[i] = Some(t); i }
            None => { self.textures.push(Some(t)); self.textures.len() - 1 }
        };
        let id = TextureId(TextureIdInner::Managed(id));
        if let TextureSource::Bytes(bytes) = source {
            self.texture_update_part(id, 0, 0, params.width as i32, params.height as i32, bytes);
        } else if let TextureSource::Array(_) = source {
            panic!("array texture sources are not supported by the Vulkan backend yet");
        }
        id
    }

    /// Command buffer for a transfer touching `tex`: the upload buffer when this frame has not
    /// drawn with it yet, otherwise the frame buffer with the render pass split around it.
    unsafe fn transfer_cmd(&mut self, used_this_frame: bool) -> (vk::CommandBuffer, Option<Target>) {
        if !used_this_frame {
            return (self.upload_cmd(), None);
        }
        let resume = self.active.as_ref().map(|a| a.target);
        if resume.is_some() { self.end_pass_inner(); }
        (self.cmd, resume)
    }

    unsafe fn texture_update_part(&mut self, id: TextureId, x: i32, y: i32, w: i32, h: i32, bytes: &[u8]) {
        let (fmt, used) = { let t = self.tex(id); (t.params.format, t.used_frame == self.frame) };
        let px = match fmt { TextureFormat::RGB8 | TextureFormat::RGBA8 => 4, TextureFormat::RGBA16F => 8, TextureFormat::Alpha => 1, _ => 4 };
        let n = (w * h) as usize * px;
        let (buf, off, ptr) = self.ring_alloc(n, 16);
        if fmt == TextureFormat::RGB8 {
            let dst = std::slice::from_raw_parts_mut(ptr, n);
            for (i, c) in bytes.chunks_exact(3).take((w * h) as usize).enumerate() {
                dst[i * 4..i * 4 + 3].copy_from_slice(c);
                dst[i * 4 + 3] = 255;
            }
        } else {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, n.min(bytes.len()));
        }
        let (cmd, resume) = self.transfer_cmd(used);
        let t = self.tex(id) as *const Texture;
        let rest = resting(&*t);
        self.layout_barrier(cmd, &*t, rest, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        let region = vk::BufferImageCopy {
            buffer_offset: off,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers { aspect_mask: aspect_of((*t).format), mip_level: 0, base_array_layer: 0, layer_count: 1 },
            image_offset: vk::Offset3D { x, y, z: 0 },
            image_extent: vk::Extent3D { width: w as u32, height: h as u32, depth: 1 },
        };
        self.core.device.cmd_copy_buffer_to_image(cmd, buf, (*t).image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);
        self.layout_barrier(cmd, &*t, vk::ImageLayout::TRANSFER_DST_OPTIMAL, rest);
        if let Some(r) = resume { self.begin_target(r, None); }
    }

    unsafe fn generate_mipmaps(&mut self, id: TextureId) {
        let used = self.tex(id).used_frame == self.frame;
        let (cmd, resume) = self.transfer_cmd(used);
        let t = self.tex(id) as *const Texture;
        let t = &*t;
        let d = &self.core.device;
        let rest = resting(t);
        self.layout_barrier(cmd, t, rest, vk::ImageLayout::GENERAL);
        let (mut w, mut h) = (t.params.width as i32, t.params.height as i32);
        for level in 1..t.levels {
            let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
            let blit = vk::ImageBlit {
                src_subresource: vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: level - 1, base_array_layer: 0, layer_count: 1 },
                src_offsets: [vk::Offset3D::default(), vk::Offset3D { x: w, y: h, z: 1 }],
                dst_subresource: vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: level, base_array_layer: 0, layer_count: 1 },
                dst_offsets: [vk::Offset3D::default(), vk::Offset3D { x: nw, y: nh, z: 1 }],
            };
            d.cmd_blit_image(cmd, t.image, vk::ImageLayout::GENERAL, t.image, vk::ImageLayout::GENERAL, &[blit], vk::Filter::LINEAR);
            self.full_barrier(cmd);
            w = nw;
            h = nh;
        }
        self.layout_barrier(cmd, t, vk::ImageLayout::GENERAL, rest);
        if let Some(r) = resume { self.begin_target(r, None); }
    }

    unsafe fn sampler(&mut self, p: &TextureParams) -> vk::Sampler {
        let key = SamplerKey(p.min_filter as u8, p.mag_filter as u8, p.mipmap_filter as u8, p.wrap as u8, 0);
        if let Some(s) = self.samplers.get(&key) { return *s; }
        let f = |m: FilterMode| if m == FilterMode::Linear { vk::Filter::LINEAR } else { vk::Filter::NEAREST };
        let wrap = match p.wrap { TextureWrap::Repeat => vk::SamplerAddressMode::REPEAT, TextureWrap::Mirror => vk::SamplerAddressMode::MIRRORED_REPEAT, TextureWrap::Clamp => vk::SamplerAddressMode::CLAMP_TO_EDGE };
        let (mipmode, maxlod) = match p.mipmap_filter {
            MipmapFilterMode::None => (vk::SamplerMipmapMode::NEAREST, 0.25),
            MipmapFilterMode::Nearest => (vk::SamplerMipmapMode::NEAREST, vk::LOD_CLAMP_NONE),
            MipmapFilterMode::Linear => (vk::SamplerMipmapMode::LINEAR, vk::LOD_CLAMP_NONE),
        };
        let s = vk_check(
            self.core.device.create_sampler(
                &vk::SamplerCreateInfo::default()
                    .min_filter(f(p.min_filter))
                    .mag_filter(f(p.mag_filter))
                    .mipmap_mode(mipmode)
                    .address_mode_u(wrap)
                    .address_mode_v(wrap)
                    .address_mode_w(wrap)
                    .max_lod(maxlod),
                None,
            ),
            "sampler",
        );
        self.samplers.insert(key, s);
        s
    }

    unsafe fn begin_target(&mut self, target: Target, action: Option<PassAction>) {
        let (key, fb, w, h) = match target {
            Target::Default => {
                let t = self.tex(self.default_color);
                (self.default_key, self.default_fb, t.params.width, t.params.height)
            }
            Target::Pass(i) => { let p = &self.passes[i]; (p.key, p.fb, p.w, p.h) }
        };
        let (cc, cd, cs) = match action {
            Some(PassAction::Clear { color, depth, stencil }) => (color, depth, stencil),
            _ => (None, None, None),
        };
        let has_depth = key.depth != vk::Format::UNDEFINED;
        let has_stencil = has_depth && aspect_of(key.depth).contains(vk::ImageAspectFlags::STENCIL);
        let v = RpVariant { key, clear_color: cc.is_some(), clear_depth: cd.is_some() && has_depth, clear_stencil: cs.is_some() && has_stencil };
        let rp = self.render_pass(v);
        let mut clears = vec![];
        let c = cc.unwrap_or((0., 0., 0., 0.));
        for _ in 0..key.ncolors { clears.push(vk::ClearValue { color: vk::ClearColorValue { float32: [c.0, c.1, c.2, c.3] } }); }
        if has_depth { clears.push(vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: cd.unwrap_or(1.0), stencil: cs.unwrap_or(0) as u32 } }); }
        self.core.device.cmd_begin_render_pass(
            self.cmd,
            &vk::RenderPassBeginInfo::default().render_pass(rp).framebuffer(fb).render_area(vk::Rect2D { offset: Default::default(), extent: vk::Extent2D { width: w, height: h } }).clear_values(&clears),
            vk::SubpassContents::INLINE,
        );
        // Mark the attachments as used this frame (later updates must split the pass).
        let frame = self.frame;
        let ids: Vec<TextureId> = match target {
            Target::Default => vec![self.default_color, self.default_depth],
            Target::Pass(i) => { let p = &self.passes[i]; p.colors.iter().copied().chain(p.depth).collect() }
        };
        for id in ids { self.tex_mut(id).used_frame = frame; }
        self.active = Some(Active { target, rp_key: key, fb, w, h });
    }

    unsafe fn end_pass_inner(&mut self) {
        if self.active.take().is_some() {
            self.core.device.cmd_end_render_pass(self.cmd);
        }
    }

    unsafe fn build_pipeline(&mut self, pid: usize, key: RpKey) -> vk::Pipeline {
        if let Some(p) = self.pipelines[pid].built.get(&key) { return *p; }
        let rp = self.render_pass(RpVariant { key, clear_color: false, clear_depth: false, clear_stencil: false });
        let pd = &self.pipelines[pid];
        let sh = &self.shaders[pd.shader.0];
        let entry = CString::new("main").unwrap();
        let stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(sh.vs).name(&entry),
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(sh.fs).name(&entry),
        ];
        // Vertex input, with miniquad's offset rules.
        let mut bindings = vec![];
        for (i, l) in pd.layouts.iter().enumerate() {
            let mut stride = l.stride as u32;
            if stride == 0 {
                stride = pd.attributes.iter().filter(|a| a.buffer_index == i).map(|a| a.format.size_bytes() as u32).sum();
            }
            bindings.push(vk::VertexInputBindingDescription {
                binding: i as u32,
                stride,
                input_rate: if l.step_func == VertexStep::PerInstance { vk::VertexInputRate::INSTANCE } else { vk::VertexInputRate::VERTEX },
            });
        }
        let mut offsets = vec![0u32; pd.layouts.len()];
        let mut attrs = vec![];
        for a in &pd.attributes {
            let off = offsets[a.buffer_index];
            if let Some(&loc) = sh.attributes.get(a.name) {
                if a.format == VertexFormat::Mat4 {
                    for k in 0..4 { attrs.push(vk::VertexInputAttributeDescription { location: loc + k, binding: a.buffer_index as u32, format: vk::Format::R32G32B32A32_SFLOAT, offset: off + 16 * k }); }
                } else {
                    attrs.push(vk::VertexInputAttributeDescription { location: loc, binding: a.buffer_index as u32, format: vertex_format(a.format, a.gl_pass_as_float), offset: off });
                }
            }
            offsets[a.buffer_index] += a.format.size_bytes() as u32;
        }
        let vi = vk::PipelineVertexInputStateCreateInfo::default().vertex_binding_descriptions(&bindings).vertex_attribute_descriptions(&attrs);
        let mut pp = pd.params;
        // Debug switches for bisecting rendering differences against GL.
        if dbg_flag("MQVK_NO_CULL") { pp.cull_face = CullFace::Nothing; }
        if dbg_flag("MQVK_NO_DEPTH") { pp.depth_test = Comparison::Always; }
        if dbg_flag("MQVK_NO_BLEND") { pp.color_blend = None; pp.alpha_blend = None; }
        let p = &pp;
        let topo = match p.primitive_type { PrimitiveType::Triangles => vk::PrimitiveTopology::TRIANGLE_LIST, PrimitiveType::Lines => vk::PrimitiveTopology::LINE_LIST, PrimitiveType::Points => vk::PrimitiveTopology::POINT_LIST };
        let ia = vk::PipelineInputAssemblyStateCreateInfo::default().topology(topo);
        let vp = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
        let mut rs = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(match p.cull_face { CullFace::Nothing => vk::CullModeFlags::NONE, CullFace::Front => vk::CullModeFlags::FRONT, CullFace::Back => vk::CullModeFlags::BACK })
            // Not flipping Y inverts the winding (see module docs).
            .front_face(match p.front_face_order { FrontFaceOrder::CounterClockwise => vk::FrontFace::CLOCKWISE, FrontFaceOrder::Clockwise => vk::FrontFace::COUNTER_CLOCKWISE });
        if let Some((factor, units)) = p.depth_write_offset {
            rs = rs.depth_bias_enable(true).depth_bias_slope_factor(factor).depth_bias_constant_factor(units);
        }
        let ms = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let st = |f: &StencilFaceState| vk::StencilOpState {
            fail_op: stencil_op(f.fail_op),
            pass_op: stencil_op(f.pass_op),
            depth_fail_op: stencil_op(f.depth_fail_op),
            compare_op: compare_func(f.test_func),
            compare_mask: f.test_mask,
            write_mask: f.write_mask,
            reference: f.test_ref as u32,
        };
        let has_depth = key.depth != vk::Format::UNDEFINED;
        let mut ds = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(has_depth && p.depth_test != Comparison::Always || has_depth && p.depth_write)
            .depth_write_enable(has_depth && p.depth_write)
            .depth_compare_op(comparison(p.depth_test));
        if let (Some(s), true) = (p.stencil_test, has_depth) {
            ds = ds.stencil_test_enable(true).front(st(&s.front)).back(st(&s.back));
        }
        let mut cba = vk::PipelineColorBlendAttachmentState::default().color_write_mask({
            let (r, g, b, a) = p.color_write;
            let mut m = vk::ColorComponentFlags::empty();
            if r { m |= vk::ColorComponentFlags::R } if g { m |= vk::ColorComponentFlags::G } if b { m |= vk::ColorComponentFlags::B } if a { m |= vk::ColorComponentFlags::A }
            m
        });
        if let Some(c) = p.color_blend {
            let a = p.alpha_blend.unwrap_or(c);
            cba = cba
                .blend_enable(true)
                .src_color_blend_factor(blend_factor(c.sfactor))
                .dst_color_blend_factor(blend_factor(c.dfactor))
                .color_blend_op(blend_op(c.equation))
                .src_alpha_blend_factor(blend_factor(a.sfactor))
                .dst_alpha_blend_factor(blend_factor(a.dfactor))
                .alpha_blend_op(blend_op(a.equation));
        }
        let cbas = vec![cba; key.ncolors as usize];
        let cb = vk::PipelineColorBlendStateCreateInfo::default().attachments(&cbas);
        let dyns = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR, vk::DynamicState::DEPTH_BIAS];
        let dy = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dyns[..if p.depth_write_offset.is_some() { 3 } else { 2 }]);
        let ci = [vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vi)
            .input_assembly_state(&ia)
            .viewport_state(&vp)
            .rasterization_state(&rs)
            .multisample_state(&ms)
            .depth_stencil_state(&ds)
            .color_blend_state(&cb)
            .dynamic_state(&dy)
            .layout(sh.layout)
            .render_pass(rp)];
        let pl = match self.core.device.create_graphics_pipelines(vk::PipelineCache::null(), &ci, None) {
            Ok(v) => v[0],
            Err((_, e)) => panic!("{}", format!("vkCreateGraphicsPipelines: {e:?}")),
        };
        self.pipelines[pid].built.insert(key, pl);
        pl
    }

    unsafe fn draw(&mut self, base: i32, count: i32, instances: i32) {
        if count <= 0 || instances <= 0 { return; }
        let Some(pipeline) = self.pipeline else { return };
        let Some(active) = self.active.as_ref() else { return };
        let (key, w, h) = (active.rp_key, active.w, active.h);
        let mut pid = pipeline.0;
        if dbg_flag("MQVK_TRACE") {
            let p = &self.pipelines[pid].params;
            eprintln!("  draw pip{pid} n={count} depth={:?}/{} ovr={:?} cull={:?} blend={} vp={:?} sc={:?}", p.depth_test, p.depth_write, self.depth_write_override, p.cull_face, p.color_blend.is_some(), self.viewport, self.scissor);
        }
        // glDepthMask override from the engine: use a twin pipeline with depth writes changed.
        if let Some(dw) = self.depth_write_override {
            if self.pipelines[pid].params.depth_write != dw {
                pid = self.override_twin(pid, dw);
            }
        }
        let sid = self.pipelines[pid].shader.0;
        // Uniforms (std140) into the ring.
        let ubo = if self.shaders[sid].ubo_size > 0 {
            let size = self.shaders[sid].ubo_size;
            let (b, off, ptr) = self.ring_alloc(size, self.ubo_align);
            let dst = std::slice::from_raw_parts_mut(ptr, size);
            dst.fill(0);
            for &(so, dof, n, _) in &self.shaders[sid].std140 {
                if so + n <= self.uniforms.len() { dst[dof..dof + n].copy_from_slice(&self.uniforms[so..so + n]); }
            }
            Some((b, off, size))
        } else { None };
        // ring_alloc may have flushed and restarted the pass; read state again.
        let active = self.active.as_ref().unwrap();
        let _ = active;
        let pl = self.build_pipeline(pid, key);
        let d = self.core.device.clone();
        let cmd = self.cmd;
        d.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pl);
        let (vx, vy, vw, vh) = self.viewport;
        d.cmd_set_viewport(cmd, 0, &[vk::Viewport { x: vx as f32, y: vy as f32, width: vw.max(1) as f32, height: vh.max(1) as f32, min_depth: 0.0, max_depth: 1.0 }]);
        let (sx, sy, sw, sh) = self.scissor;
        let (sx, sy) = (sx.max(0), sy.max(0));
        let sw = (sw.min(w as i32 - sx)).max(0);
        let sh = (sh.min(h as i32 - sy)).max(0);
        d.cmd_set_scissor(cmd, 0, &[vk::Rect2D { offset: vk::Offset2D { x: sx, y: sy }, extent: vk::Extent2D { width: sw as u32, height: sh as u32 } }]);
        if let Some((f, u)) = self.pipelines[pid].params.depth_write_offset { d.cmd_set_depth_bias(cmd, u, 0.0, f); }
        // Vertex and index buffers.
        for (i, b) in self.vbufs.iter().enumerate() {
            let b = &self.buffers[b.0];
            d.cmd_bind_vertex_buffers(cmd, i as u32, &[b.current.0], &[b.current.1]);
        }
        let Some(ib) = self.ibuf else { return };
        let ib = &self.buffers[ib.0];
        d.cmd_bind_index_buffer(cmd, ib.current.0, ib.current.1, ib.index_type);
        // Descriptors.
        let sh_ = &self.shaders[sid];
        let mut buf_infos = vec![];
        let mut img_infos = vec![];
        if let Some((b, off, size)) = ubo { buf_infos.push(vk::DescriptorBufferInfo { buffer: b, offset: off, range: size as u64 }); }
        let frame = self.frame;
        let nimg = sh_.images.min(self.images.len());
        let imgs: Vec<TextureId> = self.images[..nimg].to_vec();
        for id in imgs {
            let (view, params, depth) = { let t = self.tex(id); (t.view, t.params, t.depth) };
            let s = self.sampler(&params);
            self.tex_mut(id).used_frame = frame;
            img_infos.push((
                vk::DescriptorImageInfo { sampler: vk::Sampler::null(), image_view: view, image_layout: if depth { vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL } else { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL } },
                vk::DescriptorImageInfo { sampler: s, image_view: vk::ImageView::null(), image_layout: vk::ImageLayout::UNDEFINED },
            ));
        }
        let sh_ = &self.shaders[sid];
        let mut writes = vec![];
        if !buf_infos.is_empty() {
            writes.push(vk::WriteDescriptorSet::default().dst_binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).buffer_info(&buf_infos[..1]));
        }
        for (i, (img, smp)) in img_infos.iter().enumerate() {
            writes.push(vk::WriteDescriptorSet::default().dst_binding(1 + 2 * i as u32).descriptor_type(vk::DescriptorType::SAMPLED_IMAGE).image_info(std::slice::from_ref(img)));
            writes.push(vk::WriteDescriptorSet::default().dst_binding(2 + 2 * i as u32).descriptor_type(vk::DescriptorType::SAMPLER).image_info(std::slice::from_ref(smp)));
        }
        if !writes.is_empty() {
            self.push.cmd_push_descriptor_set(cmd, vk::PipelineBindPoint::GRAPHICS, sh_.layout, 0, &writes);
        }
        d.cmd_draw_indexed(cmd, count as u32, instances as u32, base as u32, 0, 0);
    }

    unsafe fn override_twin(&mut self, pid: usize, depth_write: bool) -> usize {
        // Twins are cached as extra pipelines keyed by (pid, flag) in a side table.
        thread_local! { static TWINS: RefCell<HashMap<(usize, bool), usize>> = RefCell::new(HashMap::new()); }
        if let Some(t) = TWINS.with(|t| t.borrow().get(&(pid, depth_write)).copied()) {
            if self.pipelines.resources.contains_key(&t) { return t; }
        }
        let src = &self.pipelines[pid];
        let mut params = src.params;
        params.depth_write = depth_write;
        let twin = PipelineData { shader: src.shader, layouts: src.layouts.clone(), attributes: src.attributes.clone(), params, built: HashMap::new() };
        let t = self.pipelines.add(twin);
        TWINS.with(|tw| tw.borrow_mut().insert((pid, depth_write), t));
        t
    }
}

/// Debug switches (MQVK_TRACE, MQVK_NO_CULL, MQVK_NO_DEPTH, MQVK_NO_BLEND), read once.
fn dbg_flag(name: &str) -> bool {
    static FLAGS: std::sync::OnceLock<[bool; 4]> = std::sync::OnceLock::new();
    let f = FLAGS.get_or_init(|| ["MQVK_TRACE", "MQVK_NO_CULL", "MQVK_NO_DEPTH", "MQVK_NO_BLEND"].map(|k| std::env::var_os(k).is_some()));
    match name { "MQVK_TRACE" => f[0], "MQVK_NO_CULL" => f[1], "MQVK_NO_DEPTH" => f[2], _ => f[3] }
}

fn resting(t: &Texture) -> vk::ImageLayout {
    if t.depth { vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL } else { vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL }
}

fn aspect_of(f: vk::Format) -> vk::ImageAspectFlags {
    match f {
        vk::Format::D32_SFLOAT | vk::Format::D16_UNORM | vk::Format::X8_D24_UNORM_PACK32 => vk::ImageAspectFlags::DEPTH,
        vk::Format::D24_UNORM_S8_UINT | vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D16_UNORM_S8_UINT => vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
        _ => vk::ImageAspectFlags::COLOR,
    }
}

fn vertex_format(f: VertexFormat, as_float: bool) -> vk::Format {
    use vk::Format as F;
    match (f, as_float) {
        (VertexFormat::Float1, _) => F::R32_SFLOAT,
        (VertexFormat::Float2, _) => F::R32G32_SFLOAT,
        (VertexFormat::Float3, _) => F::R32G32B32_SFLOAT,
        (VertexFormat::Float4, _) | (VertexFormat::Mat4, _) => F::R32G32B32A32_SFLOAT,
        (VertexFormat::Byte1, true) => F::R8_USCALED,
        (VertexFormat::Byte2, true) => F::R8G8_USCALED,
        (VertexFormat::Byte3, true) => F::R8G8B8_USCALED,
        (VertexFormat::Byte4, true) => F::R8G8B8A8_USCALED,
        (VertexFormat::Byte1, false) => F::R8_UINT,
        (VertexFormat::Byte2, false) => F::R8G8_UINT,
        (VertexFormat::Byte3, false) => F::R8G8B8_UINT,
        (VertexFormat::Byte4, false) => F::R8G8B8A8_UINT,
        (VertexFormat::Short1, true) => F::R16_USCALED,
        (VertexFormat::Short2, true) => F::R16G16_USCALED,
        (VertexFormat::Short3, true) => F::R16G16B16_USCALED,
        (VertexFormat::Short4, true) => F::R16G16B16A16_USCALED,
        (VertexFormat::Short1, false) => F::R16_UINT,
        (VertexFormat::Short2, false) => F::R16G16_UINT,
        (VertexFormat::Short3, false) => F::R16G16B16_UINT,
        (VertexFormat::Short4, false) => F::R16G16B16A16_UINT,
        (VertexFormat::Int1, _) => F::R32_UINT,
        (VertexFormat::Int2, _) => F::R32G32_UINT,
        (VertexFormat::Int3, _) => F::R32G32B32_UINT,
        (VertexFormat::Int4, _) => F::R32G32B32A32_UINT,
    }
}

fn comparison(c: Comparison) -> vk::CompareOp {
    match c {
        Comparison::Never => vk::CompareOp::NEVER,
        Comparison::Less => vk::CompareOp::LESS,
        Comparison::LessOrEqual => vk::CompareOp::LESS_OR_EQUAL,
        Comparison::Greater => vk::CompareOp::GREATER,
        Comparison::GreaterOrEqual => vk::CompareOp::GREATER_OR_EQUAL,
        Comparison::Equal => vk::CompareOp::EQUAL,
        Comparison::NotEqual => vk::CompareOp::NOT_EQUAL,
        Comparison::Always => vk::CompareOp::ALWAYS,
    }
}

fn compare_func(c: CompareFunc) -> vk::CompareOp {
    match c {
        CompareFunc::Always => vk::CompareOp::ALWAYS,
        CompareFunc::Never => vk::CompareOp::NEVER,
        CompareFunc::Less => vk::CompareOp::LESS,
        CompareFunc::Equal => vk::CompareOp::EQUAL,
        CompareFunc::LessOrEqual => vk::CompareOp::LESS_OR_EQUAL,
        CompareFunc::Greater => vk::CompareOp::GREATER,
        CompareFunc::NotEqual => vk::CompareOp::NOT_EQUAL,
        CompareFunc::GreaterOrEqual => vk::CompareOp::GREATER_OR_EQUAL,
    }
}

fn stencil_op(o: StencilOp) -> vk::StencilOp {
    match o {
        StencilOp::Keep => vk::StencilOp::KEEP,
        StencilOp::Zero => vk::StencilOp::ZERO,
        StencilOp::Replace => vk::StencilOp::REPLACE,
        StencilOp::IncrementClamp => vk::StencilOp::INCREMENT_AND_CLAMP,
        StencilOp::DecrementClamp => vk::StencilOp::DECREMENT_AND_CLAMP,
        StencilOp::Invert => vk::StencilOp::INVERT,
        StencilOp::IncrementWrap => vk::StencilOp::INCREMENT_AND_WRAP,
        StencilOp::DecrementWrap => vk::StencilOp::DECREMENT_AND_WRAP,
    }
}

fn blend_factor(f: BlendFactor) -> vk::BlendFactor {
    use vk::BlendFactor as B;
    match f {
        BlendFactor::Zero => B::ZERO,
        BlendFactor::One => B::ONE,
        BlendFactor::Value(BlendValue::SourceColor) => B::SRC_COLOR,
        BlendFactor::Value(BlendValue::SourceAlpha) => B::SRC_ALPHA,
        BlendFactor::Value(BlendValue::DestinationColor) => B::DST_COLOR,
        BlendFactor::Value(BlendValue::DestinationAlpha) => B::DST_ALPHA,
        BlendFactor::OneMinusValue(BlendValue::SourceColor) => B::ONE_MINUS_SRC_COLOR,
        BlendFactor::OneMinusValue(BlendValue::SourceAlpha) => B::ONE_MINUS_SRC_ALPHA,
        BlendFactor::OneMinusValue(BlendValue::DestinationColor) => B::ONE_MINUS_DST_COLOR,
        BlendFactor::OneMinusValue(BlendValue::DestinationAlpha) => B::ONE_MINUS_DST_ALPHA,
        BlendFactor::SourceAlphaSaturate => B::SRC_ALPHA_SATURATE,
    }
}

fn blend_op(e: Equation) -> vk::BlendOp {
    match e { Equation::Add => vk::BlendOp::ADD, Equation::Subtract => vk::BlendOp::SUBTRACT, Equation::ReverseSubtract => vk::BlendOp::REVERSE_SUBTRACT }
}

/// std140 placement of miniquad's tightly packed uniforms: (src, dst, bytes, stride) per element.
fn std140_layout(uniforms: &[UniformDesc]) -> (Vec<(usize, usize, usize, usize)>, usize) {
    let mut out = vec![];
    let (mut src, mut dst) = (0usize, 0usize);
    for u in uniforms {
        let size = u.uniform_type.size();
        let align = match u.uniform_type {
            UniformType::Float1 | UniformType::Int1 => 4,
            UniformType::Float2 | UniformType::Int2 => 8,
            _ => 16,
        };
        if u.array_count > 1 {
            let stride = (size + 15) / 16 * 16;
            dst = (dst + 15) / 16 * 16;
            for _ in 0..u.array_count {
                out.push((src, dst, size, stride));
                src += size;
                dst += stride;
            }
        } else {
            dst = (dst + align - 1) / align * align;
            out.push((src, dst, size, size));
            src += size;
            dst += size;
        }
    }
    (out, (dst + 15) / 16 * 16)
}

impl RenderingBackend for VulkanContext {
    fn info(&self) -> ContextInfo { self.0.borrow().info.clone() }

    fn new_shader(&mut self, shader: ShaderSource, meta: ShaderMeta) -> Result<ShaderId, ShaderError> {
        let (vertex, fragment) = match shader {
            ShaderSource::Glsl { vertex, fragment } => (vertex, fragment),
            _ => panic!("Metal source on the Vulkan backend"),
        };
        let uniforms: Vec<glsl::Uniform> = meta.uniforms.uniforms.iter().map(|u| glsl::Uniform { name: u.name.clone(), ty: format!("{:?}", u.uniform_type), count: u.array_count }).collect();
        let t = glsl::translate(vertex, fragment, &uniforms, &meta.images).map_err(|e| ShaderError::CompilationError { shader_type: ShaderType::Vertex, error_message: e })?;
        let mut s = self.0.borrow_mut();
        let d = s.core.device.clone();
        unsafe {
            let module = |code: &[u32]| vk_check(d.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(code), None), "shader module");
            let vs = module(&t.vertex_spv);
            let fs = module(&t.fragment_spv);
            let all = vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT;
            let mut binds = vec![];
            if !meta.uniforms.uniforms.is_empty() {
                binds.push(vk::DescriptorSetLayoutBinding::default().binding(0).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).descriptor_count(1).stage_flags(all));
            }
            for i in 0..meta.images.len() as u32 {
                binds.push(vk::DescriptorSetLayoutBinding::default().binding(1 + 2 * i).descriptor_type(vk::DescriptorType::SAMPLED_IMAGE).descriptor_count(1).stage_flags(all));
                binds.push(vk::DescriptorSetLayoutBinding::default().binding(2 + 2 * i).descriptor_type(vk::DescriptorType::SAMPLER).descriptor_count(1).stage_flags(all));
            }
            let set_layout = vk_check(
                d.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().flags(vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR).bindings(&binds), None),
                "set layout",
            );
            let sls = [set_layout];
            let layout = vk_check(d.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default().set_layouts(&sls), None), "pipeline layout");
            let (std140, ubo_size) = std140_layout(&meta.uniforms.uniforms);
            let id = s.shaders.add(Shader {
                vs, fs, set_layout, layout,
                uniforms: meta.uniforms.uniforms.clone(),
                std140,
                ubo_size: if meta.uniforms.uniforms.is_empty() { 0 } else { ubo_size },
                images: meta.images.len(),
                attributes: t.attributes.into_iter().collect(),
            });
            Ok(ShaderId(id))
        }
    }

    fn new_texture(&mut self, access: TextureAccess, source: TextureSource, params: TextureParams) -> TextureId {
        unsafe { self.0.borrow_mut().new_texture(access, source, params, None) }
    }

    fn texture_params(&self, texture: TextureId) -> TextureParams { self.0.borrow().tex(texture).params }

    unsafe fn texture_raw_id(&self, _texture: TextureId) -> RawId { RawId::OpenGl(0) }

    fn texture_set_min_filter(&mut self, texture: TextureId, filter: FilterMode, mipmap_filter: MipmapFilterMode) {
        let mut s = self.0.borrow_mut();
        let t = s.tex_mut(texture);
        t.params.min_filter = filter;
        t.params.mipmap_filter = mipmap_filter;
    }
    fn texture_set_mag_filter(&mut self, texture: TextureId, filter: FilterMode) { self.0.borrow_mut().tex_mut(texture).params.mag_filter = filter; }
    fn texture_set_wrap(&mut self, texture: TextureId, wrap_x: TextureWrap, _wrap_y: TextureWrap) { self.0.borrow_mut().tex_mut(texture).params.wrap = wrap_x; }

    fn texture_generate_mipmaps(&mut self, texture: TextureId) {
        let mut s = self.0.borrow_mut();
        if s.tex(texture).levels > 1 { unsafe { s.generate_mipmaps(texture) } }
    }

    fn texture_resize(&mut self, texture: TextureId, width: u32, height: u32, bytes: Option<&[u8]>) {
        // Recreate in place: same id, new image.
        let mut s = self.0.borrow_mut();
        let mut params = s.tex(texture).params;
        params.width = width;
        params.height = height;
        unsafe {
            let tmp = s.new_texture(TextureAccess::RenderTarget, bytes.map_or(TextureSource::Empty, TextureSource::Bytes), params, None);
            let TextureIdInner::Managed(ti) = tmp.0 else { unreachable!() };
            let TextureIdInner::Managed(i) = texture.0 else { panic!() };
            let new = s.textures[ti].take();
            let old = std::mem::replace(&mut s.textures[i], new);
            if let Some(old) = old { s.trash_texture(old); }
        }
    }

    fn texture_read_pixels(&mut self, texture: TextureId, bytes: &mut [u8]) {
        let mut s = self.0.borrow_mut();
        unsafe { s.read_pixels(texture, bytes) }
    }

    fn texture_update_part(&mut self, texture: TextureId, x_offset: i32, y_offset: i32, width: i32, height: i32, bytes: &[u8]) {
        unsafe { self.0.borrow_mut().texture_update_part(texture, x_offset, y_offset, width, height, bytes) }
    }

    fn new_render_pass_mrt(&mut self, color_img: &[TextureId], _resolve_img: Option<&[TextureId]>, depth_img: Option<TextureId>) -> RenderPass {
        let mut s = self.0.borrow_mut();
        let mut key = RpKey { colors: [vk::Format::UNDEFINED; 4], ncolors: color_img.len() as u8, depth: vk::Format::UNDEFINED };
        let mut views = vec![];
        let (mut w, mut h) = (u32::MAX, u32::MAX);
        for (i, c) in color_img.iter().enumerate() {
            let t = s.tex(*c);
            key.colors[i] = t.format;
            views.push(t.view);
            w = w.min(t.params.width);
            h = h.min(t.params.height);
        }
        if let Some(dt) = depth_img {
            let t = s.tex(dt);
            key.depth = t.format;
            views.push(t.view);
            w = w.min(t.params.width);
            h = h.min(t.params.height);
        }
        unsafe {
            let rp = s.render_pass(RpVariant { key, clear_color: false, clear_depth: false, clear_stencil: false });
            let fb = vk_check(
                s.core.device.create_framebuffer(&vk::FramebufferCreateInfo::default().render_pass(rp).attachments(&views).width(w).height(h).layers(1), None),
                "framebuffer",
            );
            RenderPass(s.passes.add(Pass { colors: color_img.to_vec(), depth: depth_img, fb, key, w, h }))
        }
    }

    fn render_pass_color_attachments(&self, render_pass: RenderPass) -> &[TextureId] {
        // Lives as long as the pass; the RefCell borrow cannot express that, so leak a copy.
        let v = self.0.borrow().passes[render_pass.0].colors.clone();
        Box::leak(v.into_boxed_slice())
    }

    fn delete_render_pass(&mut self, render_pass: RenderPass) {
        let mut s = self.0.borrow_mut();
        let p = s.passes.remove(render_pass.0);
        s.trash.push(Box::new(move |d, _| unsafe { d.destroy_framebuffer(p.fb, None) }));
    }

    fn new_pipeline(&mut self, buffer_layout: &[BufferLayout], attributes: &[VertexAttribute], shader: ShaderId, params: PipelineParams) -> Pipeline {
        let mut s = self.0.borrow_mut();
        Pipeline(s.pipelines.add(PipelineData { shader, layouts: buffer_layout.to_vec(), attributes: attributes.to_vec(), params, built: HashMap::new() }))
    }

    fn apply_pipeline(&mut self, pipeline: &Pipeline) { self.0.borrow_mut().pipeline = Some(*pipeline); }

    fn delete_pipeline(&mut self, pipeline: Pipeline) {
        let mut s = self.0.borrow_mut();
        let p = s.pipelines.remove(pipeline.0);
        let built: Vec<vk::Pipeline> = p.built.into_values().collect();
        s.trash.push(Box::new(move |d, _| unsafe { for p in built { d.destroy_pipeline(p, None) } }));
    }

    fn new_buffer(&mut self, type_: BufferType, usage: BufferUsage, data: BufferSource) -> BufferId {
        let mut s = self.0.borrow_mut();
        let (size, elem, ptr) = match &data {
            BufferSource::Slice(a) => (a.size, a.element_size, a.ptr),
            BufferSource::Empty { size, element_size } => (*size, *element_size, std::ptr::null()),
        };
        let vkusage = match type_ { BufferType::VertexBuffer => vk::BufferUsageFlags::VERTEX_BUFFER, BufferType::IndexBuffer => vk::BufferUsageFlags::INDEX_BUFFER };
        unsafe {
            let (buf, mem) = s.make_buffer(size, vkusage);
            if !ptr.is_null() { std::ptr::copy_nonoverlapping(ptr as *const u8, mem.ptr, size); }
            let cur = (buf, 0, mem.ptr);
            let index_type = if elem == 4 { vk::IndexType::UINT32 } else { vk::IndexType::UINT16 };
            BufferId(s.buffers.add(Buffer { buf, mem, size, index_type, dynamic: usage != BufferUsage::Immutable, current: cur, in_ring_len: 0 }))
        }
    }

    fn buffer_update(&mut self, buffer: BufferId, data: BufferSource) {
        let mut s = self.0.borrow_mut();
        let BufferSource::Slice(a) = data else { return };
        let size = a.size.min(s.buffers[buffer.0].size);
        unsafe {
            let (b, off, ptr) = s.ring_alloc(size, 16);
            std::ptr::copy_nonoverlapping(a.ptr as *const u8, ptr, size);
            let buf = &mut s.buffers[buffer.0];
            buf.current = (b, off, ptr);
            buf.in_ring_len = size;
            s.dynamic_in_ring.push(buffer.0);
        }
    }

    fn buffer_size(&mut self, buffer: BufferId) -> usize { self.0.borrow().buffers[buffer.0].size }

    fn delete_buffer(&mut self, buffer: BufferId) {
        let mut s = self.0.borrow_mut();
        let b = s.buffers.remove(buffer.0);
        s.trash.push(Box::new(move |d, a| unsafe { d.destroy_buffer(b.buf, None); a.free(d, b.mem) }));
    }

    fn delete_texture(&mut self, texture: TextureId) {
        let mut s = self.0.borrow_mut();
        let TextureIdInner::Managed(i) = texture.0 else { return };
        if let Some(t) = s.textures[i].take() { unsafe { s.trash_texture(t) } }
    }

    fn delete_shader(&mut self, program: ShaderId) {
        let mut s = self.0.borrow_mut();
        let sh = s.shaders.remove(program.0);
        s.trash.push(Box::new(move |d, _| unsafe {
            d.destroy_shader_module(sh.vs, None);
            d.destroy_shader_module(sh.fs, None);
            d.destroy_pipeline_layout(sh.layout, None);
            d.destroy_descriptor_set_layout(sh.set_layout, None);
        }));
    }

    fn apply_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) { self.0.borrow_mut().viewport = (x, y, w, h); }
    fn apply_scissor_rect(&mut self, x: i32, y: i32, w: i32, h: i32) { self.0.borrow_mut().scissor = (x, y, w, h); }

    fn apply_bindings_from_slice(&mut self, vertex_buffers: &[BufferId], index_buffer: BufferId, textures: &[TextureId]) {
        let mut s = self.0.borrow_mut();
        s.vbufs = vertex_buffers.to_vec();
        s.ibuf = Some(index_buffer);
        s.images = textures.to_vec();
    }

    fn apply_uniforms_from_bytes(&mut self, uniform_ptr: *const u8, size: usize) {
        let mut s = self.0.borrow_mut();
        s.uniforms.clear();
        s.uniforms.extend_from_slice(unsafe { std::slice::from_raw_parts(uniform_ptr, size) });
    }

    fn clear(&mut self, color: Option<(f32, f32, f32, f32)>, depth: Option<f32>, stencil: Option<i32>) {
        let s = self.0.borrow();
        if dbg_flag("MQVK_TRACE") { eprintln!("  clear {color:?} {depth:?} {stencil:?} active={}", s.active.is_some()); }
        if s.active.is_none() {
            // GL clears whatever is bound, which outside a pass is the default framebuffer
            // (still limited by the scissor). Open the default target with LOAD, clear, close.
            drop(s);
            let mut m = self.0.borrow_mut();
            unsafe { m.begin_target(Target::Default, None) };
            drop(m);
            self.clear(color, depth, stencil);
            unsafe { self.0.borrow_mut().end_pass_inner() };
            return;
        }
        let Some(a) = s.active.as_ref() else { return };
        let mut atts = vec![];
        if let Some(c) = color {
            for i in 0..a.rp_key.ncolors as u32 {
                atts.push(vk::ClearAttachment { aspect_mask: vk::ImageAspectFlags::COLOR, color_attachment: i, clear_value: vk::ClearValue { color: vk::ClearColorValue { float32: [c.0, c.1, c.2, c.3] } } });
            }
        }
        if a.rp_key.depth != vk::Format::UNDEFINED && (depth.is_some() || stencil.is_some()) {
            let mut m = vk::ImageAspectFlags::empty();
            if depth.is_some() { m |= vk::ImageAspectFlags::DEPTH; }
            if stencil.is_some() && aspect_of(a.rp_key.depth).contains(vk::ImageAspectFlags::STENCIL) { m |= vk::ImageAspectFlags::STENCIL; }
            if !m.is_empty() {
                atts.push(vk::ClearAttachment { aspect_mask: m, color_attachment: 0, clear_value: vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: depth.unwrap_or(1.0), stencil: stencil.unwrap_or(0) as u32 } } });
            }
        }
        if atts.is_empty() { return; }
        let (sx, sy, sw, sh) = s.scissor;
        let (sx, sy) = (sx.max(0), sy.max(0));
        let w = (sw.min(a.w as i32 - sx)).max(0) as u32;
        let h = (sh.min(a.h as i32 - sy)).max(0) as u32;
        if w == 0 || h == 0 { return; }
        let rect = [vk::ClearRect { rect: vk::Rect2D { offset: vk::Offset2D { x: sx, y: sy }, extent: vk::Extent2D { width: w, height: h } }, base_array_layer: 0, layer_count: 1 }];
        unsafe { s.core.device.cmd_clear_attachments(s.cmd, &atts, &rect) }
    }

    fn begin_default_pass(&mut self, action: PassAction) { self.begin_pass(None, action) }

    fn begin_pass(&mut self, pass: Option<RenderPass>, action: PassAction) {
        let mut s = self.0.borrow_mut();
        if dbg_flag("MQVK_TRACE") {
            let d = pass.map(|p| { let p = &s.passes[p.0]; format!("pass{} {}x{} depth={}", 0, p.w, p.h, p.depth.is_some()) }).unwrap_or("default".into());
            let a = match &action { PassAction::Nothing => "Nothing".to_string(), PassAction::Clear { color, depth, stencil } => format!("Clear({color:?},{depth:?},{stencil:?})") };
            eprintln!("begin {d} {a}");
        }
        unsafe {
            s.end_pass_inner();
            let target = match pass { None => Target::Default, Some(p) => Target::Pass(p.0) };
            s.begin_target(target, Some(action));
            let a = s.active.as_ref().unwrap();
            let (w, h) = (a.w as i32, a.h as i32);
            s.viewport = (0, 0, w, h);
            s.scissor = (0, 0, w, h);
        }
    }

    fn end_render_pass(&mut self) { unsafe { self.0.borrow_mut().end_pass_inner() } }

    fn commit_frame(&mut self) {
        if dbg_flag("MQVK_TRACE") { eprintln!("commit"); }
        let mut s = self.0.borrow_mut();
        unsafe {
            s.end_pass_inner();
            s.submit(true);
        }
        s.frame += 1;
        s.pipeline = None;
    }

    fn draw(&self, base_element: i32, num_elements: i32, num_instances: i32) {
        unsafe { self.0.borrow_mut().draw(base_element, num_elements, num_instances) }
    }

    fn set_depth_write_override(&mut self, depth_write: Option<bool>) { self.0.borrow_mut().depth_write_override = depth_write; }

    fn texture_grab_screen(&mut self, texture: TextureId) {
        let mut s = self.0.borrow_mut();
        unsafe { s.grab_screen(texture) }
    }
}

impl Inner {
    unsafe fn trash_texture(&mut self, t: Texture) {
        self.trash.push(Box::new(move |d, a| {
            d.destroy_image_view(t.view, None);
            d.destroy_image(t.image, None);
            a.free(d, t.mem);
        }));
    }

    unsafe fn read_pixels(&mut self, id: TextureId, bytes: &mut [u8]) {
        let (w, h, fmt) = { let t = self.tex(id); (t.params.width, t.params.height, t.params.format) };
        let px = if fmt == TextureFormat::RGBA16F { 8 } else if fmt == TextureFormat::Alpha { 1 } else { 4 };
        let n = (w * h) as usize * px;
        let (buf, mem) = self.make_buffer(n, vk::BufferUsageFlags::TRANSFER_DST);
        let (cmd, resume) = self.transfer_cmd(true);
        let t = self.tex(id) as *const Texture;
        let rest = resting(&*t);
        self.layout_barrier(cmd, &*t, rest, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
        let region = vk::BufferImageCopy {
            buffer_offset: 0, buffer_row_length: 0, buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers { aspect_mask: aspect_of((*t).format), mip_level: 0, base_array_layer: 0, layer_count: 1 },
            image_offset: vk::Offset3D::default(),
            image_extent: vk::Extent3D { width: w, height: h, depth: 1 },
        };
        self.core.device.cmd_copy_image_to_buffer(cmd, (*t).image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, buf, &[region]);
        self.layout_barrier(cmd, &*t, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, rest);
        self.full_barrier(cmd);
        self.submit(false);
        let src = std::slice::from_raw_parts(mem.ptr, n);
        if fmt == TextureFormat::RGB8 {
            for (i, p) in src.chunks_exact(4).enumerate() { if i * 3 + 3 <= bytes.len() { bytes[i * 3..i * 3 + 3].copy_from_slice(&p[..3]); } }
        } else {
            let m = n.min(bytes.len());
            bytes[..m].copy_from_slice(&src[..m]);
        }
        self.core.device.destroy_buffer(buf, None);
        let d = self.core.device.clone();
        self.alloc.free(&d, mem);
        if let Some(r) = resume { self.begin_target(r, None); }
    }

    /// glCopyTexImage2D from the current target's first colour attachment.
    unsafe fn grab_screen(&mut self, dst: TextureId) {
        let src = match self.active.as_ref().map(|a| a.target) {
            Some(Target::Pass(i)) => self.passes[i].colors[0],
            _ => self.default_color,
        };
        let (cmd, resume) = self.transfer_cmd(true);
        let (s, d) = (self.tex(src) as *const Texture, self.tex(dst) as *const Texture);
        let (s, d) = (&*s, &*d);
        let w = s.params.width.min(d.params.width);
        let h = s.params.height.min(d.params.height);
        self.layout_barrier(cmd, s, resting(s), vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
        self.layout_barrier(cmd, d, resting(d), vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        let sub = vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 };
        if s.format == d.format {
            self.core.device.cmd_copy_image(cmd, s.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, d.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[vk::ImageCopy { src_subresource: sub, src_offset: Default::default(), dst_subresource: sub, dst_offset: Default::default(), extent: vk::Extent3D { width: w, height: h, depth: 1 } }]);
        } else {
            let o = [vk::Offset3D::default(), vk::Offset3D { x: w as i32, y: h as i32, z: 1 }];
            self.core.device.cmd_blit_image(cmd, s.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, d.image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[vk::ImageBlit { src_subresource: sub, src_offsets: o, dst_subresource: sub, dst_offsets: o }], vk::Filter::NEAREST);
        }
        self.layout_barrier(cmd, s, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, resting(s));
        self.layout_barrier(cmd, d, vk::ImageLayout::TRANSFER_DST_OPTIMAL, resting(d));
        self.tex_mut(dst).used_frame = self.frame;
        if let Some(r) = resume { self.begin_target(r, None); }
    }
}

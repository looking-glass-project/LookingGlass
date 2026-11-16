//! Backend conformance test: renders the same frame with GL or Vulkan and dumps
//! the offscreen target and a screen grab as PPM. MINIQUAD_BACKEND=vulkan selects Vulkan.
use miniquad::*;

#[repr(C)] struct V { pos: [f32; 3], uv: [f32; 2], col: [u8; 4] }

const VS: &str = "#version 100
attribute vec3 pos; attribute vec2 uv; attribute vec4 col;
uniform mat4 mvp; uniform vec2 offset;
varying lowp vec4 color; varying highp vec2 tc;
void main() { gl_Position = mvp * vec4(pos.xy + offset, pos.z, 1.0); color = col / 255.0; tc = uv; }";
const FS: &str = "#version 100
precision mediump float;
varying lowp vec4 color; varying highp vec2 tc;
uniform sampler2D tex; uniform float mixv; uniform float arr[3];
void main() { vec4 t = texture2D(tex, tc); gl_FragColor = mix(color, t, mixv) * vec4(arr[0], arr[1], arr[2], 1.0); }";

#[repr(C)] struct U { mvp: [f32; 16], offset: [f32; 2], mixv: f32, arr: [f32; 3] }

struct Stage { ctx: Box<dyn RenderingBackend>, pip: Pipeline, pip_blend: Pipeline, tri: Bindings, quad: Bindings, pass: RenderPass, rt: TextureId, grab: TextureId, frame: u32 }

fn ident() -> [f32; 16] { [1.,0.,0.,0., 0.,1.,0.,0., 0.,0.,1.,0., 0.,0.,0.,1.] }

impl Stage {
    fn new() -> Stage {
        let mut ctx = window::new_rendering_backend();
        eprintln!("backend: {}", ctx.info().gl_version_string);
        let shader = ctx.new_shader(ShaderSource::Glsl { vertex: VS, fragment: FS }, ShaderMeta {
            images: vec!["tex".into()],
            uniforms: UniformBlockLayout { uniforms: vec![
                UniformDesc::new("mvp", UniformType::Mat4), UniformDesc::new("offset", UniformType::Float2),
                UniformDesc::new("mixv", UniformType::Float1), UniformDesc::array(UniformDesc::new("arr", UniformType::Float1), 3)] },
        }).unwrap();
        let layout = [BufferLayout::default()];
        let attrs = [VertexAttribute::new("pos", VertexFormat::Float3), VertexAttribute::new("uv", VertexFormat::Float2),
                     VertexAttribute::with_buffer("col", VertexFormat::Byte4, 0)];
        let mut attrs = attrs.to_vec(); attrs[2].gl_pass_as_float = true;
        let pip = ctx.new_pipeline(&layout, &attrs, shader, PipelineParams { depth_test: Comparison::LessOrEqual, depth_write: true, cull_face: CullFace::Back, front_face_order: FrontFaceOrder::CounterClockwise, ..Default::default() });
        let pip_blend = ctx.new_pipeline(&layout, &attrs, shader, PipelineParams {
            color_blend: Some(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::OneMinusValue(BlendValue::SourceAlpha))),
            ..Default::default() });
        // A triangle in front of a second one (depth test), as dynamic buffer updated twice per frame.
        let tri_v = [
            V { pos: [-0.8, -0.8, 0.5], uv: [0., 0.], col: [255, 0, 0, 255] }, V { pos: [0.8, -0.8, 0.5], uv: [1., 0.], col: [0, 255, 0, 255] }, V { pos: [0.0, 0.8, 0.5], uv: [0.5, 1.], col: [0, 0, 255, 255] },
            V { pos: [-0.5, -0.2, 0.2], uv: [0., 0.], col: [255, 255, 0, 255] }, V { pos: [0.5, -0.2, 0.2], uv: [1., 0.], col: [255, 255, 0, 255] }, V { pos: [0.0, 0.3, 0.9], uv: [0.5, 1.], col: [255, 255, 0, 255] },
        ];
        let tvb = ctx.new_buffer(BufferType::VertexBuffer, BufferUsage::Stream, BufferSource::empty::<V>(6));
        ctx.buffer_update(tvb, BufferSource::slice(&tri_v));
        let tib = ctx.new_buffer(BufferType::IndexBuffer, BufferUsage::Immutable, BufferSource::slice(&[0u16, 1, 2, 3, 4, 5]));
        // checker texture
        let mut px = vec![0u8; 8 * 8 * 4];
        for y in 0..8 { for x in 0..8 { let i = (y * 8 + x) * 4; let on = (x + y) % 2 == 0; px[i..i + 4].copy_from_slice(if on { &[255, 255, 255, 128] } else { &[0, 0, 0, 255] }); } }
        let checker = ctx.new_texture_from_rgba8(8, 8, &px);
        ctx.texture_set_filter(checker, FilterMode::Nearest, MipmapFilterMode::None);
        let rt = ctx.new_render_texture(TextureParams { width: 64, height: 48, ..Default::default() });
        let depth = ctx.new_render_texture(TextureParams { width: 64, height: 48, format: TextureFormat::Depth, ..Default::default() });
        let pass = ctx.new_render_pass(rt, Some(depth));
        let grab = ctx.new_render_texture(TextureParams { width: 96, height: 64, ..Default::default() });
        let quad_v = [
            V { pos: [-0.9, -0.9, 0.], uv: [0., 0.], col: [255; 4] }, V { pos: [0.1, -0.9, 0.], uv: [1., 0.], col: [255; 4] },
            V { pos: [0.1, 0.1, 0.], uv: [1., 1.], col: [255; 4] }, V { pos: [-0.9, 0.1, 0.], uv: [0., 1.], col: [255; 4] },
        ];
        let qvb = ctx.new_buffer(BufferType::VertexBuffer, BufferUsage::Immutable, BufferSource::slice(&quad_v));
        let qib = ctx.new_buffer(BufferType::IndexBuffer, BufferUsage::Immutable, BufferSource::slice(&[0u16, 1, 2, 0, 2, 3]));
        Stage { pip, pip_blend, tri: Bindings { vertex_buffers: vec![tvb], index_buffer: tib, images: vec![checker] },
                quad: Bindings { vertex_buffers: vec![qvb], index_buffer: qib, images: vec![rt] }, pass, rt, grab, frame: 0, ctx }
    }
}

fn dump(ctx: &mut Box<dyn RenderingBackend>, t: TextureId, name: &str) {
    let (w, h) = ctx.texture_size(t);
    let mut b = vec![0u8; (w * h * 4) as usize];
    ctx.texture_read_pixels(t, &mut b);
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    for y in (0..h).rev() { for x in 0..w { let i = ((y * w + x) * 4) as usize; out.extend_from_slice(&b[i..i + 3]); } }
    std::fs::write(name, out).unwrap();
}

impl EventHandler for Stage {
    fn update(&mut self) {}
    fn draw(&mut self) {
        let ctx = &mut self.ctx;
        // offscreen: depth-tested triangles, mixed 30% with the checker
        ctx.begin_pass(Some(self.pass), PassAction::clear_color(0.1, 0.2, 0.6, 1.0));
        ctx.apply_pipeline(&self.pip);
        ctx.apply_bindings(&self.tri);
        ctx.apply_uniforms(UniformsSource::table(&U { mvp: ident(), offset: [0., 0.], mixv: 0.3, arr: [1., 1., 1.] }));
        ctx.draw(0, 6, 1);
        ctx.end_render_pass();
        // default target: grey, render texture on a quad (tinted via the uniform array), then the
        // triangle again half transparent, shifted, after a second buffer update in the same frame.
        ctx.begin_default_pass(PassAction::clear_color(0.5, 0.5, 0.5, 1.0));
        ctx.apply_pipeline(&self.pip);
        ctx.apply_bindings(&self.quad);
        ctx.apply_uniforms(UniformsSource::table(&U { mvp: ident(), offset: [0., 0.], mixv: 1.0, arr: [1.0, 0.8, 0.6] }));
        ctx.draw(0, 6, 1);
        ctx.apply_pipeline(&self.pip_blend);
        ctx.apply_bindings(&self.tri);
        ctx.apply_scissor_rect(10, 5, 70, 50);
        ctx.apply_uniforms(UniformsSource::table(&U { mvp: ident(), offset: [0.4, 0.3], mixv: 0.0, arr: [1., 1., 1.] }));
        ctx.draw(0, 3, 1);
        ctx.texture_grab_screen(self.grab);
        ctx.end_render_pass();
        ctx.commit_frame();
        self.frame += 1;
        if self.frame == 2 {
            let tag = std::env::var("MINIQUAD_BACKEND").unwrap_or("gl".into());
            dump(ctx, self.rt, &format!("rt_{tag}.ppm"));
            dump(ctx, self.grab, &format!("grab_{tag}.ppm"));
            window::order_quit();
        }
    }
}

fn main() {
    miniquad::start(conf::Conf { window_width: 96, window_height: 64, ..Default::default() }, || Box::new(Stage::new()));
}

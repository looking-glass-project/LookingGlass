//! Rewrites miniquad's GLSL ES 1.00 shaders as Vulkan GLSL 4.50 and compiles them
//! to SPIR-V with naga (pure Rust, so it also runs on the PS5).
//!
//! Conventions of the generated code (the Vulkan backend must follow them):
//! - every non-sampler uniform goes into one std140 block, set 0 binding 0, named
//!   `_U`, members in the order of the miniquad `ShaderMeta` (the vertex and the
//!   fragment stage share it);
//! - image `i` of `ShaderMeta::images` is a sampled image at set 0 binding 1+2i and its
//!   sampler at binding 2+2i (naga has no combined image samplers);
//! - vertex attributes get locations in order of declaration; `attr_names` returns them;
//! - varyings get locations by name, identically in both stages;
//! - clip space is converted to Vulkan's depth range (0..1); Y is left as in GL. The backend
//!   renders every target in GL's memory layout (row 0 = bottom) and flips only on present.

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Uniform { pub name: String, pub ty: String, pub count: usize }

#[derive(Debug)]
pub struct Translated {
    pub vertex_glsl: String,
    pub fragment_glsl: String,
    pub vertex_spv: Vec<u32>,
    pub fragment_spv: Vec<u32>,
    /// attribute name -> location
    pub attributes: Vec<(String, u32)>,
}

fn strip_comments(src: &str) -> String {
    let mut out = String::new();
    let b = src.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' { i += 1; }
        } else if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') { i += 1; }
            i += 2;
        } else { out.push(b[i] as char); i += 1; }
    }
    out
}

/// Splits the source into top-level statements terminated by ';' (outside braces) and
/// brace blocks; returns (declarations we consume, remaining code).
struct Parsed { uniforms: Vec<(String, String, usize)>, samplers: Vec<String>, attributes: Vec<(String, String)>, varyings: Vec<(String, String)>, rest: String }

fn parse(src: &str) -> Parsed {
    let src = strip_comments(src);
    let mut p = Parsed { uniforms: vec![], samplers: vec![], attributes: vec![], varyings: vec![], rest: String::new() };
    let mut depth = 0i32;
    let mut stmt = String::new();
    for c in src.chars() {
        stmt.push(c);
        match c {
            '{' => depth += 1,
            '}' => { depth -= 1; if depth == 0 { p.rest.push_str(&stmt); stmt.clear(); } }
            ';' if depth == 0 => { handle(&stmt, &mut p); stmt.clear(); }
            '\n' if depth == 0 && stmt.trim_start().starts_with('#') => {
                let t = stmt.trim();
                if !t.starts_with("#version") && !t.starts_with("#extension") { p.rest.push_str(&stmt); }
                stmt.clear();
            }
            _ => {}
        }
    }
    p.rest.push_str(&stmt);
    p
}

fn clean_type(words: &[&str]) -> Vec<String> {
    words.iter().filter(|w| !matches!(**w, "highp" | "mediump" | "lowp" | "flat")).map(|s| s.to_string()).collect()
}

fn handle(stmt: &str, p: &mut Parsed) {
    let s = stmt.trim().trim_end_matches(';').trim();
    let words: Vec<&str> = s.split_whitespace().collect();
    let kind = words.first().copied().unwrap_or("");
    if !matches!(kind, "uniform" | "attribute" | "varying") || s.contains('(') {
        if kind == "precision" { return; }
        p.rest.push_str(stmt);
        return;
    }
    let w = clean_type(&words[1..]);
    let ty = w[0].clone();
    let names = w[1..].join(" ");
    for n in names.split(',') {
        let n = n.trim();
        if n.is_empty() { continue; }
        let (name, count) = match n.find('[') {
            Some(i) => (n[..i].trim().to_string(), n[i + 1..n.find(']').unwrap()].trim().parse().unwrap_or(1)),
            None => (n.to_string(), 1),
        };
        match kind {
            "uniform" if ty.starts_with("sampler") => p.samplers.push(name),
            "uniform" => p.uniforms.push((ty.clone(), name, count)),
            "attribute" => p.attributes.push((ty.clone(), name)),
            _ => p.varyings.push((ty.clone(), name)),
        }
    }
}

fn rename_main(code: &str) -> String {
    // the only `void main(` in valid GLSL
    let i = code.find("void main").expect("shader has no main");
    format!("{}void _user_main{}", &code[..i], &code[i + "void main".len()..])
}

fn replace_ident(code: &str, from: &str, to: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let b = code.as_bytes();
    let mut i = 0;
    let is_id = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    while i < b.len() {
        if code[i..].starts_with(from)
            && (i == 0 || !is_id(b[i - 1]))
            && (i + from.len() >= b.len() || !is_id(b[i + from.len()]))
        {
            out.push_str(to);
            i += from.len();
        } else {
            out.push(b[i] as char);
            i += 1;
        }
    }
    out
}

pub fn translate(vs: &str, fs: &str, uniforms: &[Uniform], images: &[String]) -> Result<Translated, String> {
    let v = parse(vs);
    let f = parse(fs);
    // One shared uniform block, in meta order; type text comes from the source.
    let mut types: BTreeMap<String, String> = BTreeMap::new();
    for (ty, n, _) in v.uniforms.iter().chain(f.uniforms.iter()) { types.insert(n.clone(), ty.clone()); }
    let mut block = if uniforms.is_empty() { String::new() } else { String::from("layout(std140, set = 0, binding = 0) uniform _U {\n") };
    for u in uniforms {
        let ty = types.get(&u.name).cloned().unwrap_or_else(|| meta_type(&u.ty).to_string());
        if u.count > 1 { block += &format!("  {} {}[{}];\n", ty, u.name, u.count); } else { block += &format!("  {} {};\n", ty, u.name); }
    }
    if !uniforms.is_empty() { block += "};\n"; }
    let mut samplers = String::new();
    for (i, n) in images.iter().enumerate() {
        // naga has no combined image samplers: image i = texture at 1+2i, sampler at 2+2i
        samplers += &format!("layout(set = 0, binding = {}) uniform texture2D {n}_t;\nlayout(set = 0, binding = {}) uniform sampler {n}_s;\n", 1 + 2 * i, 2 + 2 * i);
    }
    // Varyings by name, union of both stages.
    let mut vary: BTreeMap<String, String> = BTreeMap::new();
    for (ty, n) in v.varyings.iter().chain(f.varyings.iter()) { vary.insert(n.clone(), ty.clone()); }
    let loc: BTreeMap<String, usize> = vary.keys().enumerate().map(|(i, k)| (k.clone(), i)).collect();

    let header = "#version 450\n";
    let common = |code: &str| {
        let mut c = replace_ident(code, "texture2D", "texture");
        c = replace_ident(&c, "texture2DLod", "textureLod");
        c = replace_ident(&c, "textureCube", "texture");
        for q in ["highp", "mediump", "lowp"] { c = replace_ident(&c, q, ""); }
        for n in images { c = replace_ident(&c, n, &format!("sampler2D({n}_t, {n}_s)")); }
        c
    };

    // Vertex
    let mut vg = String::from(header);
    vg += &block; vg += &samplers;
    let mut attributes = vec![];
    let mut next = 0u32;
    for (ty, n) in v.attributes.iter() {
        vg += &format!("layout(location = {}) in {} {};\n", next, ty, n);
        attributes.push((n.clone(), next));
        next += match ty.as_str() { "mat4" => 4, "mat3" => 3, "mat2" => 2, _ => 1 };
    }
    for (ty, n) in &v.varyings { vg += &format!("layout(location = {}) out {} {};\n", loc[n], ty, n); }
    vg += &rename_main(&common(&v.rest));
    vg += "\nvoid main() {\n  _user_main();\n  gl_Position.z = (gl_Position.z + gl_Position.w) * 0.5;\n}\n";

    // Fragment
    let mut fg = String::from(header);
    fg += &block; fg += &samplers;
    for (ty, n) in &f.varyings { fg += &format!("layout(location = {}) in {} {};\n", loc[n], ty, n); }
    fg += "layout(location = 0) out vec4 _frag_color;\n";
    let mut fr = common(&f.rest);
    fr = replace_ident(&fr, "gl_FragColor", "_frag_color");
    fr = fr.replace("gl_FragData[0]", "_frag_color");
    fg += &fr;

    let vertex_spv = compile(&vg, naga::ShaderStage::Vertex).map_err(|e| format!("vertex: {e}\n---\n{vg}"))?;
    let fragment_spv = compile(&fg, naga::ShaderStage::Fragment).map_err(|e| format!("fragment: {e}\n---\n{fg}"))?;
    Ok(Translated { vertex_glsl: vg, fragment_glsl: fg, vertex_spv, fragment_spv, attributes })
}

fn meta_type(t: &str) -> &'static str {
    match t { "Float1" => "float", "Float2" => "vec2", "Float3" => "vec3", "Float4" => "vec4",
              "Int1" => "int", "Int2" => "ivec2", "Int3" => "ivec3", "Int4" => "ivec4", "Mat4" => "mat4", _ => "vec4" }
}

pub fn compile(src: &str, stage: naga::ShaderStage) -> Result<Vec<u32>, String> {
    let mut front = naga::front::glsl::Frontend::default();
    let module = front
        .parse(&naga::front::glsl::Options::from(stage), src)
        .map_err(|e| e.emit_to_string(src))?;
    let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .map_err(|e| e.emit_to_string(src))?;
    {
        // naga flips gl_Position.y for Vulkan by default; we keep GL's orientation on purpose.
        let mut opts = naga::back::spv::Options::default();
        opts.flags.remove(naga::back::spv::WriterFlags::ADJUST_COORDINATE_SPACE);
        naga::back::spv::write_vec(&module, &info, &opts, None)
    }
        .map_err(|e| e.to_string())
}

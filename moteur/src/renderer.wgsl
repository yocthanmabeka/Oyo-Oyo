// Chaque point lumineux est un carré de deux triangles, dessiné une fois par instance.
// Le fragment en fait un disque avec un halo, mélangé en addition sur fond noir.

struct Uniforms {
    aspect: f32,
    time: f32,
    reserve0: f32,
    reserve1: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct Instance {
    @location(0) position: vec2<f32>,
    @location(1) radius: f32,
    @location(2) color: vec4<f32>,
};

struct Output {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32, inst: Instance) -> Output {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    );
    let corner = corners[vi];
    // Le halo dépasse le disque : on dessine un carré 1,6 fois plus large que le rayon.
    let margin = 1.6;
    let p = vec2<f32>(inst.position.x / u.aspect, inst.position.y)
          + corner * vec2<f32>(inst.radius * margin / u.aspect, inst.radius * margin);
    var s: Output;
    s.clip = vec4<f32>(p, 0.0, 1.0);
    s.uv = corner * margin;
    s.color = inst.color;
    return s;
}

@fragment
fn fs(s: Output) -> @location(0) vec4<f32> {
    let d = length(s.uv);
    let core = smoothstep(1.0, 0.55, d);          // le disque, bord doux
    let halo = exp(-d * d * 2.2) * 0.55;           // la lumière autour
    let a = clamp(core + halo, 0.0, 1.0) * s.color.a;
    return vec4<f32>(s.color.rgb * a, a);
}

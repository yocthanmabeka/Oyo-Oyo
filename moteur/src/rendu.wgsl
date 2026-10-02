// Chaque point lumineux est un carré de deux triangles, dessiné une fois par instance.
// Le fragment en fait un disque avec un halo, mélangé en addition sur fond noir.

struct Uniformes {
    aspect: f32,
    temps: f32,
    reserve0: f32,
    reserve1: f32,
};

@group(0) @binding(0) var<uniform> u: Uniformes;

struct Instance {
    @location(0) position: vec2<f32>,
    @location(1) rayon: f32,
    @location(2) couleur: vec4<f32>,
};

struct Sortie {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) couleur: vec4<f32>,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32, inst: Instance) -> Sortie {
    var coins = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    );
    let coin = coins[vi];
    // Le halo dépasse le disque : on dessine un carré 1,6 fois plus large que le rayon.
    let marge = 1.6;
    let p = vec2<f32>(inst.position.x / u.aspect, inst.position.y)
          + coin * vec2<f32>(inst.rayon * marge / u.aspect, inst.rayon * marge);
    var s: Sortie;
    s.clip = vec4<f32>(p, 0.0, 1.0);
    s.uv = coin * marge;
    s.couleur = inst.couleur;
    return s;
}

@fragment
fn fs(s: Sortie) -> @location(0) vec4<f32> {
    let d = length(s.uv);
    let noyau = smoothstep(1.0, 0.55, d);          // le disque, bord doux
    let halo = exp(-d * d * 2.2) * 0.55;           // la lumière autour
    let a = clamp(noyau + halo, 0.0, 1.0) * s.couleur.a;
    return vec4<f32>(s.couleur.rgb * a, a);
}

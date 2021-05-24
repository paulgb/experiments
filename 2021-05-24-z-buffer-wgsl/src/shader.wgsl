struct VertexOutput {
    [[location(0)]] coord: vec2<f32>;
    [[location(1)]] c: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Locals {
    time: f32;
    radius: f32;
};
[[group(0), binding(0)]]
var r_locals: Locals;

let corners: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1., -1.),
    vec2<f32>(1., -1.),
    vec2<f32>(-1., 1.),
    vec2<f32>(1., -1.),
    vec2<f32>(-1., 1.),
    vec2<f32>(1., 1.),
);

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32,
    [[builtin(instance_index)]] in_instance_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    let u_radius = r_locals.radius;
    let u_time = r_locals.time;

    let speed: f32 = (fract(sin(f32(in_instance_index)+1.) * 99999.0) - 0.5) / 100.;
    let r_: f32 = fract(sin(f32(in_instance_index)+1.) * 99998.0);
    let r: f32 = (1. - r_ * r_) * (1. - u_radius);
    let x: f32 = r * cos(speed * u_time);
    let y: f32 = r * sin(speed * u_time);

    out.coord = corners[in_vertex_index];
    
    out.position = vec4<f32>(x + u_radius * out.coord.x, y + u_radius * out.coord.y, 0., 1.);

    return out;
}

let ITERATIONS: i32 = 50;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let cx: f32 = (in.c.x - 0.5) * 1.2;
    let cy: f32 = in.c.y * 1.2;

    var zx: f32 = in.coord.x * 2.;
    var zy: f32 = in.coord.y * 2.;

    for(var i: i32 = 0; i < ITERATIONS; i = i + 1) {
        let xtemp: f32 = (zx * zx) - (zy * zy);
        zy = (2. * zx * zy) + cy;
        zx = xtemp + cx;

        if ((zx * zx) + (zy * zy) > 4.) {
            if (i < 1) {
                discard;
            }

            let frac: f32 = f32(i) / f32(ITERATIONS);
            return vec4<f32>(12. * frac, 1.5 * frac, 3. * frac, 1.0);
        }
    }

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
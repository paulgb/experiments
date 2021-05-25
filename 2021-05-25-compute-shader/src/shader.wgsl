[[block]]
struct InputData {
    data: [[stride(4)]] array<f32>;
};

[[group(0), binding(0)]]
var<storage> v_data: [[access(read_write)]] InputData;

let ITERATIONS: u32 = 1u;

[[stage(compute), workgroup_size(1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    var x: f32 = 0.;
    var y: f32 = 0.;

    let xtemp = y;
    y = 5.;
    x = xtemp;

    v_data.data[0] = x;
}
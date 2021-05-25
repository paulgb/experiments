[[block]]
struct InputData {
    data: [[stride(4)]] array<u32>;
};

[[group(0), binding(0)]]
var<storage> v_data: [[access(read_write)]] InputData;

[[stage(compute), workgroup_size(1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    v_data.data[global_id.x] = 7u;
}
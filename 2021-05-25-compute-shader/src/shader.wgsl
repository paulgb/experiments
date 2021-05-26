[[block]]
struct IOData {
    data: f32;
};

[[group(0), binding(0)]]
var<storage> v_data: [[access(read_write)]] IOData;

[[stage(compute), workgroup_size(1)]]
fn main() {
    var a: f32 = 0.1;
    let b: f32 = a;
    a = 4.1;
    v_data.data = b;
}
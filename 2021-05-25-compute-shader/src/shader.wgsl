[[block]]
struct IOData {
    data: f32;
};

[[group(0), binding(0)]]
var<storage> v_data: [[access(read_write)]] IOData;

[[stage(compute), workgroup_size(1)]]
fn main() {
    var x: f32 = 0.1;
    var y: f32 = 0.2;

    let xtemp: f32 = y;
    y = 4.1;
    x = xtemp;

    v_data.data = x;
}
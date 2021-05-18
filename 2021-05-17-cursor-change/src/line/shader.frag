#version 450

layout(location=0) in vec4 v_color;
layout(location=1) in vec2 v_edge;

layout(location=0) out vec4 f_color;

void main() {
    if (v_edge.x < fwidth(v_edge.x)) {
        f_color = vec4(1., 0., 0., 1.);
    } else {
        f_color = v_color;
    }
}

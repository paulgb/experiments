#version 450

layout(location=0) in vec2 a_position;
layout(location=1) in vec4 a_color;
layout(location=2) in float a_radius;

layout(location=0) out vec4 v_color;

void main() {
    gl_Position = vec4(a_position, 0., 1.);
    gl_PointSize = a_radius;

    v_color = a_color;
}

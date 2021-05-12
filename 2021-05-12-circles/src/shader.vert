#version 450

layout(location=0) in vec2 a_position;
layout(location=1) in vec4 a_color;
layout(location=2) in float a_radius;

layout(location=0) out vec4 v_color;
layout(location=1) out vec2 v_coord;

void main() {
    switch (gl_VertexIndex) {
        case 0:
            gl_Position = vec4(a_position.x - a_radius, a_position.y - a_radius, 0., 1.);
            v_coord = vec2(-1., -1.);
            break;
        case 1:
        case 3:
            gl_Position = vec4(a_position.x + a_radius, a_position.y - a_radius, 0., 1.);
            v_coord = vec2(1., -1.);
            break;
        case 2:
        case 4:
            gl_Position = vec4(a_position.x - a_radius, a_position.y + a_radius, 0., 1.);
            v_coord = vec2(-1., 1.);
            break;
        case 5:
            gl_Position = vec4(a_position.x + a_radius, a_position.y + a_radius, 0., 1.);
            v_coord = vec2(1., 1.);
    }

    v_color = a_color;
}

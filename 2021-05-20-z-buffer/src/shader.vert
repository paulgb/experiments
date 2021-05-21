#version 450

layout(location=0) out vec2 v_coord;
layout(location=1) out vec2 v_c;

const float RADIUS = 0.02;

layout(set=0, binding=0)
uniform Uniforms {
    float u_time;
};

void main() {
    float speed = (fract(sin(gl_InstanceIndex+1) * 99999.0) - 0.5) / 100.;
    float r_ = fract(sin(gl_InstanceIndex+1) * 99998.0);
    float r = 1 - r_ * r_;

    float x = r * cos(speed * u_time);
    float y = r * sin(speed * u_time);

    switch (gl_VertexIndex) {
        case 0:
            gl_Position = vec4(x - RADIUS, y - RADIUS, 0, 1.);
            v_coord = vec2(-1., -1.);
            break;
        case 1:
        case 3:
            gl_Position = vec4(x + RADIUS, y - RADIUS, 0, 1.);
            v_coord = vec2(1., -1.);
            break;
        case 2:
        case 4:
            gl_Position = vec4(x - RADIUS, y + RADIUS, 0, 1.);
            v_coord = vec2(-1., 1.);
            break;
        case 5:
            gl_Position = vec4(x + RADIUS, y + RADIUS, 0, 1.);
            v_coord = vec2(1., 1.);
    }

    v_c = vec2(x, y);
}

#version 450

layout(location=0) in vec4 v_color;
layout(location=1) in vec2 v_coord;

layout(location=0) out vec4 f_color;

void main() {
    float r = dot(v_coord, v_coord);
    float delta = fwidth(r);

    /*
    float alpha = 1.0 - smoothstep(1.0 - delta*2., 1.0, r);
    f_color = vec4(v_color.rgb, v_color.a * alpha);
    */

    f_color = v_color;
}

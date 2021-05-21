#version 450

layout(location=0) in vec2 v_coord;
layout(location=1) in vec2 v_c;

layout(location=0) out vec4 f_color;

const int ITERATIONS = 20;

void main() {
    /*
    float r = dot(v_coord, v_coord);
    if (r > 1) {
        discard;
    }
    */

    float cx = (v_c.x - 0.5) * 1.2;
    float cy = v_c.y * 1.2;

    float zx = v_coord.x * 2.;
    float zy = v_coord.y * 2.;

    for (int i = 0; i < ITERATIONS; i++) {
        float xtemp = zx * zx - zy * zy;
        zy = 2 * zx * zy + cy;
        zx = xtemp + cx;

        if (zx * zx + zy * zy > 2.) {
            f_color = vec4(5. * i / ITERATIONS, 1.5 * i / ITERATIONS, 3. * i / ITERATIONS, 1.0);
            return;
        }
    }

    //f_color = vec4(0.0, 0.0, 0.0, 1.0);
    discard;
}

varying vec2 v_uv;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 a_vt;

void main() {
    v_uv = a_vt;
    gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_color_texture;
uniform sampler2D u_outline_texture;
uniform ivec2 u_outline_texture_size;

uniform vec4 u_outline_color;

void main() {
    vec4 pixel_outline = texture2D(u_outline_texture, v_uv);
    gl_FragColor = texture2D(u_color_texture, v_uv);

    vec2 outline_texture_size = vec2(u_outline_texture_size);
    float outline_thickness = outline_texture_size.y * 1.0 / 800.0;

    const int W = 1;
    for (int x = -W; x <= W; ++x) {
        for (int y = -W; y <= W; ++y) {
            vec4 this_outline = texture2D(u_outline_texture, v_uv + vec2(x, y) / float(W) * outline_thickness / outline_texture_size);
            if (length((this_outline.xyz * 2.0 - 1.0) - (pixel_outline.xyz * 2.0 - 1.0)) > 0.2
                || abs(this_outline.w - pixel_outline.w) >= 0.01) {
                gl_FragColor = u_outline_color;
                return;
            }
        }
    }
}
#endif

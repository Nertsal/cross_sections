uniform float u_time;

varying vec3 v_normal;
varying float v_cutoff;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec3 a_normal;

uniform mat4 u_model_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;

uniform mat4 u_cutoff_matrix;

void main() {
    v_normal = mat3(u_model_matrix) * a_normal;
    vec4 world_pos = u_model_matrix * vec4(a_pos, 1.0);

    vec4 cut_pos = u_cutoff_matrix * world_pos;
    v_cutoff = cut_pos.x / cut_pos.w; // Cutoff the positive x-axis
    
    vec4 pos = u_projection_matrix * u_view_matrix * world_pos;
    gl_Position = pos;
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;

void main() {
    if (v_cutoff > 0.0) {
        discard;
    }

    vec3 light_dir = normalize(vec3(1.0, 2.0, 0.5));
    float light = 0.1 + dot(light_dir, v_normal) * 0.5 + 0.5;
    gl_FragColor = u_color * light;
}
#endif

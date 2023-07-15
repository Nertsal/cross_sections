varying vec3 v_normal;
varying vec4 v_color;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec3 a_normal;
attribute vec4 a_color;

uniform mat4 u_model_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;

void main() {
    v_color = a_color;
    v_normal = normalize(vec3(u_model_matrix * vec4(a_normal, 0.0)));
    gl_Position = u_projection_matrix * u_view_matrix * u_model_matrix * vec4(a_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
    vec3 light_dir = normalize(vec3(1.0, 0.5, 2.0));
    float light_level = max(dot(v_normal, light_dir), 0.0);

    vec4 color = v_color * 0.4 + v_color * light_level * 0.6;
    gl_FragColor = color;
}
#endif

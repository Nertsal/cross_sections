varying vec3 v_normal;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec3 a_normal;

uniform mat4 u_model_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;

void main() {
    v_normal = normalize(vec3(u_model_matrix * vec4(a_normal, 0.0)));
    gl_Position = u_projection_matrix * u_view_matrix * u_model_matrix * vec4(a_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
    gl_FragColor = vec4((normalize(v_normal) * 0.5 + 0.5), gl_FragCoord.z);
}
#endif

#version 150 core

in vec2 a_Pos;
in vec2 a_Uv;
in vec3 a_Color;
out vec2 v_Uv;
out vec4 v_Color;
uniform mat4 u_Proj;

void main() {
    v_Uv = a_Uv;
    v_Color = vec4(a_Color, 1.0);
    gl_Position = u_Proj * vec4(a_Pos, 0.0, 1.0);
}

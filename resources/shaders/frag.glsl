#version 150 core

uniform sampler2D t_Texture;

in vec2 v_Uv;
in vec4 v_Color;
out vec4 Target0;

void main() {
    Target0 = v_Color * texture(t_Texture, v_Uv);
}

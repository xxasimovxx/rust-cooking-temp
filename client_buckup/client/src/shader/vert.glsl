#version 330 core
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex;

out vec4 frag_color;

void main() {
    gl_Position = vec4(pos, 1.0);
    frag_color = vec4(tex, 1.0, 1.0);
}

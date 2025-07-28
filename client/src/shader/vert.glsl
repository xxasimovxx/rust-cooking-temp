#version 330 core
uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex;

out vec2 tex_cords;

void main() {
    gl_Position = projection * view * model * vec4(pos, 1.0);
    tex_cords = tex;
}

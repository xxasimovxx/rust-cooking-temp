#version 330 core
uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex;

out vec2 tex_cords;
out vec3 normal_vec;
out vec3 frag_pos;

void main() {
    
    frag_pos = vec3(model * vec4(pos, 1.0));
    gl_Position = projection * view * vec4(frag_pos, 1.0);
    tex_cords = tex;
    normal_vec = mat3(transpose(inverse(model))) * normal;
}

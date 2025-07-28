#version 330 core
uniform sampler2D texture_image;

in vec2 tex_cords;
out vec4 final_color;

void main() {
    final_color = texture(texture_image, tex_cords);
}

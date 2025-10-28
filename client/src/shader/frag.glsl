#version 330 core
uniform sampler2D texture_image;

in vec2 tex_cords;
in vec3 normal_vec;
in vec3 frag_pos;
out vec4 final_color;

void main() {
    // lighting parameters
    float ambient_strength = 0.1;
    vec3 light_pos = vec3(0.0, 10.0, 0.0);
    vec3 light_color = vec3(1.0);

    // ambient
    vec3 ambient = ambient_strength * light_color;

    // diffuse
    vec3 norm = normalize(normal_vec);
    vec3 light_direction = normalize(light_pos - frag_pos);
    float diff = max(dot(norm, light_direction), 0.2);
    vec3 diffuse = diff * light_color;

    // combine
    vec3 result = (ambient + diffuse) * texture(texture_image, tex_cords).rgb;

    final_color = vec4(result, 1.0);
}


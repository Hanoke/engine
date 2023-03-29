#version 450

layout(location = 0) in vec3 in_frag_color;
layout(location = 1) in vec2 in_frag_uv;

layout(binding = 1) uniform sampler2D uv_sampler;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = vec4(in_frag_color * texture(uv_sampler, in_frag_uv * 3.0).rgb, 1.0);
}
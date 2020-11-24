#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UniformBufferObject {
    float u_time;
} ubo;
layout(binding = 1) uniform sampler2D texSampler;

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 color = vec3(fragColor.x, fragColor.y, abs(sin(ubo.u_time)));
    outColor = vec4(color, 1.0);
}

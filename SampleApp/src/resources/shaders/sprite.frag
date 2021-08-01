#version 450

layout (location = 0) in vec2 texCoord;

layout(set = 0, binding = 0) uniform texture2D u_Texture;
layout(set = 0, binding = 1) uniform sampler u_Sampler;

layout (location = 0) out vec4 outColor;

void main() {
	outColor = texture(sampler2D(u_Texture, u_Sampler), texCoord);
}
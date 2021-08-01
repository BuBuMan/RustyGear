#version 450

layout (location = 0) in vec3 vPosition;
layout (location = 1) in vec3 vNormal;
layout (location = 2) in vec2 vTexCoord;

layout(set = 1, binding = 0) uniform uniforms {
	mat4 view_matrix;
} Uniforms;

layout (location = 0) out vec2 texCoord;

layout(push_constant) uniform model_properties {
	mat4 model_matrix;
} ModelProperties;

void main() {
	gl_Position = Uniforms.view_matrix*ModelProperties.model_matrix*vec4(vPosition, 1.0);
	texCoord = vTexCoord;
}
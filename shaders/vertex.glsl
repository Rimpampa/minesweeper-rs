#version 130 core

in vec2 coord;
in vec2 texture_coord;
in int texture_idx;

uniform float scale = 1;
uniform vec2 offset = vec2(0);
uniform vec2 aspect = vec2(1);

out vec2 vs_TextureCoord;
flat out int vs_TextureIndex;

void main(void)
{
	vs_TextureCoord = texture_coord;
	vs_TextureIndex = texture_idx;

	gl_Position = vec4((coord + offset) * scale * aspect, 0.5, 1);
}
#version 130 core
#define discord discard

in vec2 vs_TextureCoord;
flat in int vs_TextureIndex;

uniform sampler2D texture0;
uniform sampler2D texture1;
uniform sampler2D texture2;

uniform int texture0_idx;
uniform int texture1_idx;
uniform int texture2_idx;

out vec4 fragColor;

const int SIZE = 100;

void main(void)
{
	vec4 color;
	if (vs_TextureIndex == texture0_idx)
		color = texture(texture0, vs_TextureCoord);
	
	else if (vs_TextureIndex == texture1_idx)
		color = texture(texture1, vs_TextureCoord);

	else if (vs_TextureIndex == texture2_idx)
		color = texture(texture2, vs_TextureCoord);
	else
		color = vec4(1, 0, 1, 1);
	
	if (color.a > 0.1)
		fragColor = color;
	else discord;
}
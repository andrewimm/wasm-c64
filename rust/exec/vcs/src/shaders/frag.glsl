#version 330

in vec2 v_texcoord;

uniform usampler2D screen;
uniform sampler2D palette;

out vec4 outColor;

void main() {
  uint color_index = texture(screen, v_texcoord).r;
  outColor = texture(palette, vec2(float(color_index) / 256.0, 0.5));
}
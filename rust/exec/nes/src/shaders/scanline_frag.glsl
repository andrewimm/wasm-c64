#version 330
precision highp usampler2D;

in vec2 v_texcoord;

uniform usampler2D scanline;
uniform sampler2D colors;

out vec4 outColor;

void main() {
  uint color_index = texture(scanline, v_texcoord).r;
  outColor = texture(colors, vec2(float(color_index) / 64., 0.5));
}
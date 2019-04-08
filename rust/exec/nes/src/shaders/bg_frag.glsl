#version 330
precision highp float;
precision highp usampler2D;

in vec2 v_texcoord;

uniform usampler2D pattern_0;
uniform usampler2D pattern_1;

out vec4 outColor;

void main() {
  float tile_x = floor(v_texcoord.x / 8.);
  float tile_y = floor(v_texcoord.y / 8.);
  float tile_index = tile_x + tile_y * 16.;
  float local_x = fract(v_texcoord.x / 8.) * 8.;
  float local_y = fract(v_texcoord.y / 8.);

  float pattern_loc = tile_index / 256.;

  uint low = texture(pattern_0, vec2(local_y / 2.0, pattern_loc)).r;
  uint high = texture(pattern_0, vec2((1. + local_y) / 2.0, pattern_loc)).r;
  uint shift = uint(7. - local_x);
  uint low_bit = (low >> shift) & 0x1u;
  uint high_bit = (high >> (shift - 1u)) & 0x2u;
  uint palette_index = low_bit | high_bit;
  vec4 color = vec4(0.0, 0.0, 0.0, 1.0);
  if (palette_index == 1u) {
    color = vec4(0.33, 0.33, 0.33, 1.0);
  } else if (palette_index == 2u) {
    color = vec4(0.66, 0.66, 0.66, 1.0);
  } else if (palette_index == 3u) {
    color = vec4(1.0, 1.0, 1.0, 1.0);
  }
  outColor = color;
}
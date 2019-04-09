#version 330
precision highp usampler2D;

in vec2 v_texcoord;

uniform usampler2D pattern;
uniform usampler2D nametable;
uniform usampler2D attributes;
uniform usampler2D palettes;
uniform sampler2D colors;

uniform int bgcolor;
uniform ivec3 bg_palette_0;
uniform ivec3 bg_palette_1;
uniform ivec3 bg_palette_2;
uniform ivec3 bg_palette_3;

out vec4 outColor;

void main() {
  float tile_x = v_texcoord.x / 8.;
  float tile_y = v_texcoord.y / 8.;
  float local_x = fract(v_texcoord.x / 8.) * 8.;
  float local_y = fract(v_texcoord.y / 8.);

  float pattern_loc = float(texture(nametable, vec2(tile_x / 32., tile_y / 30.))) / 255.;

  uint low = texture(pattern, vec2(local_y / 2.0, pattern_loc)).r;
  uint high = texture(pattern, vec2((1. + local_y) / 2.0, pattern_loc)).r;
  uint shift = uint(round(8. - local_x));
  uint low_bit = (low >> shift) & 0x1u;
  uint high_bit = (high >> (shift - 1u)) & 0x2u;
  uint palette_index = low_bit | high_bit;

  float attr_x = v_texcoord.x / 256.;
  float attr_y = v_texcoord.y / 256.;
  uint attr_block = texture(attributes, vec2(attr_x, attr_y)).r;

  shift = 0u;
  if ((int(tile_x) & 2) == 2) {
    shift += 2u;
  }
  if ((int(tile_y) & 2) == 2) {
    shift += 4u;
  }
  uint attr = (attr_block >> shift) & 3u;

  ivec3 palette = bg_palette_0;
  if (attr == 1u) {
    palette = bg_palette_1;
  } else if (attr == 2u) {
    palette = bg_palette_2;
  } else if (attr == 3u) {
    palette = bg_palette_3;
  }

  int color_index = bgcolor;
  if (palette_index == 1u) {
    color_index = palette.x;
  } else if (palette_index == 2u) {
    color_index = palette.y;
  } else if (palette_index == 3u) {
    color_index = palette.z;
  }
  outColor = texture(colors, vec2(float(color_index) / 64., 0.5));
}
#version 330
precision highp usampler2D;

in vec2 v_texcoord;

uniform usampler2D pattern;
uniform sampler2D colors;
uniform float tile_index;
uniform int sprite_palette;

uniform int bgcolor;
uniform ivec3 sprite_palette_0;
uniform ivec3 sprite_palette_1;
uniform ivec3 sprite_palette_2;
uniform ivec3 sprite_palette_3;

out vec4 outColor;

void main() {
  vec2 loc_low = vec2(v_texcoord.y / 2.0, tile_index / 255.);
  vec2 loc_high = vec2((1. + v_texcoord.y) / 2.0, tile_index / 255.);
  uint low = texture(pattern, loc_low).r;
  uint high = texture(pattern, loc_high).r;
  if (v_texcoord.y > 1.0) {
    loc_low = vec2((v_texcoord.y - 1.0) / 2.0, (tile_index + 1.) / 255.);
    loc_high = vec2(v_texcoord.y / 2.0, (tile_index + 1.) / 255.);
    low = texture(pattern, loc_low).r;
    high = texture(pattern, loc_high).r;
  }
  
  uint shift = uint(floor(8. - v_texcoord.x * 8.));
  uint low_bit = (low >> shift) & 0x1u;
  uint high_bit = ((high >> shift) & 0x1u) << 1;
  uint palette_index = low_bit | high_bit;

  outColor = vec4(float(palette_index) / 3., float(palette_index) / 3., float(palette_index) / 3., 1.0);

  ivec3 palette = sprite_palette_0;
  if (sprite_palette == 1) {
    palette = sprite_palette_1;
  } else if (sprite_palette == 2) {
    palette = sprite_palette_2;
  } else if (sprite_palette == 3) {
    palette = sprite_palette_3;
  }

  outColor = vec4(float(sprite_palette) / 3., float(sprite_palette) / 3., float(sprite_palette) / 3., 1.0);

  if (palette_index == 0u) {
    outColor = vec4(0, 0, 0, 0);
  } else {
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
}
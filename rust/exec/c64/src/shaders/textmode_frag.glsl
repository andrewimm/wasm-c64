#version 300 es
precision highp float;
precision highp usampler2D;

in vec2 v_texcoord;
uniform int u_framecolor;
uniform int u_bgcolor;
uniform sampler2D u_colors;
uniform usampler2D u_screenmem;
uniform usampler2D u_colormem;
uniform usampler2D u_charmem;

out vec4 outColor;

float u_charmem_length = 256.;

void main() {
  int index = u_bgcolor;
  vec2 tile = vec2(floor(v_texcoord.x / 8.), floor(v_texcoord.y / 8.));
  vec2 coords = vec2(v_texcoord.x / 8. / 40., v_texcoord.y / 8. / 25.);
  uint charindex = texture(u_screenmem, coords).r;
  int offsetx = 7 - int(v_texcoord.x - (floor(v_texcoord.x / 8.) * 8.));
  float offsety = (v_texcoord.y - (floor(v_texcoord.y / 8.) * 8.)) / 8.;
  vec2 memcoord = vec2(offsety, (float(charindex) + 0.5) / u_charmem_length);
  uint line = texture(u_charmem, memcoord).r;
  float tileIndex = tile.y * 40. + tile.x;
  if (((line >> offsetx) & 1u) > 0u) {
    index = int(texture(u_colormem, vec2(tileIndex / 1024., 0.5)).r);
  }

  if (v_texcoord.x < 0. || v_texcoord.y < 0. || v_texcoord.x > 320. || v_texcoord.y > 200.) {
    index = u_framecolor;
  }
  vec4 color = texture(u_colors, vec2(0, float(index) / 16.));

  outColor = vec4(
    color.xyz,
    1.0
  );
}
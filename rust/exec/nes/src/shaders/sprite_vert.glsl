#version 330
in vec2 a_position;

uniform float position_x;
uniform float position_y;
uniform float height_scale;
uniform int flip;

out vec2 v_texcoord;

void main() {
  vec2 scaled = a_position * 8.0;
  if ((flip & 1) == 1) {
    scaled.x = -1.0 * scaled.x + 8.0;
  }
  scaled.y *= height_scale;
  scaled.x += position_x;
  scaled.y += position_y;
  gl_Position = vec4(scaled.x / 256. - 1., 1. - scaled.y / 120., 0, 1);
  v_texcoord = vec2(a_position.x, a_position.y * height_scale);
}
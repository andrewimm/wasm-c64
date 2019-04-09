#version 330
in vec2 a_position;

uniform float position_x;
uniform float position_y;

out vec2 v_texcoord;

void main() {
  vec2 scaled = a_position * 8.0;
  scaled.x += position_x;
  scaled.y += position_y;
  gl_Position = vec4(scaled.x / 128. - 1., 1. - scaled.y / 120., 0, 1);
  v_texcoord = a_position;
}
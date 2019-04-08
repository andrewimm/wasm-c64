#version 330
in vec2 a_position;
out vec2 v_texcoord;

void main() {
  gl_Position = vec4(a_position.xy * 2.0 - vec2(1, 1), 0, 1);
  v_texcoord = vec2(a_position.x * 384. - 32., (1.0 - a_position.y) * 272. - 36.);
}
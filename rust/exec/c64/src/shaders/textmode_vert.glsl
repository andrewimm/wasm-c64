#version 300 es
in vec4 a_position;
out vec2 v_texcoord;

void main() {
  gl_Position = a_position * 2.0 - vec4(1, 1, 0, 1);
  v_texcoord = vec2(a_position.x * 384. - 32., (1.0 - a_position.y) * 272. - 36.);
}
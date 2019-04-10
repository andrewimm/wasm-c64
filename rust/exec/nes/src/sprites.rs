use gllite::program::Program;
use gllite::node::Node;
use gllite::uniforms::UniformValue;
use std::rc::Rc;
use nesmemmap::ppu::PPU;
use nesmemmap::sprite::Sprite;

pub fn build_sprite_program() -> Program {
  let shader_sprite_frag = include_str!("shaders/sprite_frag.glsl");
  let shader_sprite_vert = include_str!("shaders/sprite_vert.glsl");

  let mut sprite_program = gllite::program::Program::new();
  sprite_program
    .add_shader(shader_sprite_vert, gl::VERTEX_SHADER)
    .add_shader(shader_sprite_frag, gl::FRAGMENT_SHADER)
    .compile();
  sprite_program
}

pub fn create_sprite_meshes(p: &Rc<Program>) -> Vec<Node> {
  let mut sprites = Vec::with_capacity(64);
  for i in 0..64 {
    let mut mesh = Node::for_program(Rc::clone(p));
    mesh.add_attribute(String::from("a_position"));
    let vertices: [f32; 12] = [
      1.0, 0.0,
      0.0, 0.0,
      0.0, 1.0,
      0.0, 1.0,
      1.0, 1.0,
      1.0, 0.0,
    ];
    mesh.buffer_data(&vertices);
    sprites.push(mesh);
  }
  sprites
}

pub fn update_sprite_mesh(mesh: &mut Node, sprite: &Sprite, ppu: &PPU) {
  mesh.set_uniform(String::from("position_x"), UniformValue::Float(sprite.x_position as f32));
  // Sprites are delayed by one scanline
  mesh.set_uniform(String::from("position_y"), UniformValue::Float(sprite.y_position as f32 + 1.0));
  mesh.set_uniform(String::from("height_scale"), UniformValue::Float(if ppu.double_height_sprites { 2.0 } else { 1.0 }));
  mesh.set_uniform(String::from("sprite_palette"), UniformValue::Int(sprite.palette as i32));
  let mut flip: i32 = if sprite.flip_horizontal { 1 } else { 0 };
  if sprite.flip_vertical {
    flip = flip | 2;
  }
  mesh.set_uniform(String::from("flip"), UniformValue::Int(flip))
}
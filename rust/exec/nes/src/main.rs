#![feature(box_syntax)]

use gllite;
use gllite::gli;
use gllite::uniforms::UniformValue;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::time::{self, SystemTime};

use nesmemmap::mapper;
use mos6510::memory::Memory;
mod palette;
mod sprites;
mod vm;
use vm::VM;

fn main() {
  let mapper = load_mapper();

  let mut shell = emushell::EmuShell::with_size_and_scale(256, 240, 2);
  shell.make_active_gl_context();
  shell.set_title("Rust NES");

  let shader_bg_frag = include_str!("shaders/bg_frag.glsl");
  let shader_bg_vert = include_str!("shaders/bg_vert.glsl");

  let mut bg_program = gllite::program::Program::new();
  bg_program
    .add_shader(shader_bg_vert, gl::VERTEX_SHADER)
    .add_shader(shader_bg_frag, gl::FRAGMENT_SHADER)
    .compile();

  bg_program.make_current();
  
  let bg = Rc::new(bg_program);

  let mut screen = gllite::node::Node::for_program(Rc::clone(&bg));
  let vertices: [f32; 12] = [
    0.0, 0.0, 0.0, 1.0, 1.0, 0.0,
    1.0, 0.0, 0.0, 1.0, 1.0, 1.0,
  ];
  screen.add_attribute(String::from("a_position"));
  screen.buffer_data(&vertices);

  let mut pattern_0 = gllite::texture::Texture::new();
  pattern_0.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  pattern_0.set_filter_mode(gli::NEAREST, gli::NEAREST);
  let mut pattern_1 = gllite::texture::Texture::new();
  pattern_1.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  pattern_1.set_filter_mode(gli::NEAREST, gli::NEAREST);

  let mut nametable = gllite::texture::Texture::new();
  nametable.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  nametable.set_filter_mode(gli::NEAREST, gli::NEAREST);

  let mut attributes = gllite::texture::Texture::new();
  attributes.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  attributes.set_filter_mode(gli::NEAREST, gli::NEAREST);

  let mut colors = gllite::texture::Texture::new();
  colors.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  colors.set_filter_mode(gli::NEAREST, gli::NEAREST);
  colors.set_from_bytes(gli::RGBA, 64, 1, gli::RGBA, &palette::COLORS[0] as *const u8);

  screen.set_uniform(String::from("pattern"), pattern_0.as_uniform_value());
  screen.set_uniform(String::from("nametable"), nametable.as_uniform_value());
  screen.set_uniform(String::from("colors"), colors.as_uniform_value());
  screen.set_uniform(String::from("attributes"), attributes.as_uniform_value());

  let sprite_program = Rc::new(sprites::build_sprite_program());
  let mut sprite_meshes = sprites::create_sprite_meshes(&sprite_program);

  let mut vm = VM::new(mapper);
  vm.mem.ppu.set_scanline(241);

  let mut last_frame_time = SystemTime::now();
  loop {
    let now = SystemTime::now();
    let mut delta = match now.duration_since(last_frame_time) {
      Ok(n) => n.as_millis(),
      Err(_) => 1,
    };
    last_frame_time = now;

    if delta < 16 {
      let diff = 16 - delta;
      let sleeptime = time::Duration::from_millis(diff as u64);
      thread::sleep(sleeptime);
      delta += diff;
    }

    shell.update();
    if shell.should_exit() {
      break;
    }

    if shell.in_foreground() {
      // Run VM, draw result
      let mut total: u32 = 0;
      while total < 113 * 262 {
        let cycles = vm.step();
        vm.mem.ppu.add_cpu_cycles(cycles);
        if vm.mem.ppu.should_interrupt() {
          vm.cpu.nonmaskable_interrupt(&mut vm.mem);
        }
        total += cycles as u32;
      }

      let pattern_source = match vm.mem.ppu.background_address {
        nesmemmap::ppu::BackgroundTableAddress::Base => pattern_0.as_uniform_value(),
        nesmemmap::ppu::BackgroundTableAddress::Offset => pattern_1.as_uniform_value(),
      };
      screen.set_uniform(String::from("pattern"), pattern_source);
      screen.set_uniform(String::from("bgcolor"), UniformValue::Int(vm.mem.ppu.bg_color as i32));
      let bg_palette_0 = vm.mem.ppu.bg_palette_0;
      screen.set_uniform(String::from("bg_palette_0"), UniformValue::IntVec3(bg_palette_0.0 as i32, bg_palette_0.1 as i32, bg_palette_0.2 as i32));
      let bg_palette_1 = vm.mem.ppu.bg_palette_1;
      screen.set_uniform(String::from("bg_palette_1"), UniformValue::IntVec3(bg_palette_1.0 as i32, bg_palette_1.1 as i32, bg_palette_1.2 as i32));
      let bg_palette_2 = vm.mem.ppu.bg_palette_2;
      screen.set_uniform(String::from("bg_palette_2"), UniformValue::IntVec3(bg_palette_2.0 as i32, bg_palette_2.1 as i32, bg_palette_2.2 as i32));
      let bg_palette_3 = vm.mem.ppu.bg_palette_3;
      screen.set_uniform(String::from("bg_palette_3"), UniformValue::IntVec3(bg_palette_3.0 as i32, bg_palette_3.1 as i32, bg_palette_3.2 as i32));
      pattern_0.set_from_bytes(gli::R8UI, 16, 256, gli::RED_INTEGER, vm.mem.get_pattern_0_ptr());
      pattern_1.set_from_bytes(gli::R8UI, 16, 256, gli::RED_INTEGER, vm.mem.get_pattern_1_ptr());
      nametable.set_from_bytes(gli::R8UI, 32, 30, gli::RED_INTEGER, vm.mem.ppu.get_nametable_ptr());
      attributes.set_from_bytes(gli::R8UI, 8, 8, gli::RED_INTEGER, vm.mem.ppu.get_attribute_ptr());

      let mut i = 0;
      for mesh in sprite_meshes.iter_mut() {
        let sprite = vm.mem.ppu.get_sprite(i);
        sprites::update_sprite_mesh(mesh, sprite);
        i += 1;
      }

      unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
      }
      bg.make_current();
      screen.draw();
      sprite_program.make_current();
      for sprite in sprite_meshes.iter_mut() {
        sprite.draw();
      }
    }

    shell.swap_buffers();
  }
}

fn load_mapper() -> Box<mapper::Mapper> {
  let mut bin_seen = false;
  let mut file_seen = false;
  let mut file_name = String::new();
  for arg in env::args() {
    if !bin_seen {
      bin_seen = true;
    } else {
      file_name = arg;
      file_seen = true;
    }
  }
  if !file_seen {
    panic!("Must load a ROM file");
  }

  let path = Path::new(&file_name);
  let mut file = match File::open(&path) {
    Err(msg) => panic!("Couldn't open file: {}", msg.description()),
    Ok(file) => file,
  };
  let mut buffer = Vec::new();
  match file.read_to_end(&mut buffer) {
    Err(msg) => panic!("Couldn't read file: {}", msg.description()),
    Ok(_) => (),
  };

  mapper::create_mapper(&buffer)
}
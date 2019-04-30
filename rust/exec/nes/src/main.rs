#![feature(box_syntax)]

use glutin::{VirtualKeyCode};
use gllite;
use gllite::gli;
use gllite::uniforms::UniformValue;
use std::env;
use std::error::Error;
use std::ffi::c_void;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::time::{self, SystemTime};

mod apu;
use nesmemmap::mapper;
use nesmemmap::ppu::SpriteTableAddress;
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

  // scanline program
  let scanline_frag = include_str!("shaders/scanline_frag.glsl");
  let scanline_vert = include_str!("shaders/scanline_vert.glsl");
  let mut scanline_program = gllite::program::Program::new();
  scanline_program
    .add_shader(scanline_vert, gl::VERTEX_SHADER)
    .add_shader(scanline_frag, gl::FRAGMENT_SHADER)
    .compile();
  scanline_program.make_current();

  let scanline_prog = Rc::new(scanline_program);

  let mut scanline_screen = gllite::node::Node::for_program(Rc::clone(&scanline_prog));
  let scanline_vertices: [f32; 12] = [
    0.0, 0.0, 0.0, 1.0, 1.0, 0.0,
    1.0, 0.0, 0.0, 1.0, 1.0, 1.0,
  ];
  scanline_screen.add_attribute(String::from("a_position"));
  scanline_screen.buffer_data(&scanline_vertices);

  let mut scanline_tex = gllite::texture::Texture::new();
  scanline_tex.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  scanline_tex.set_filter_mode(gli::NEAREST, gli::NEAREST);

  let mut colors_tex = gllite::texture::Texture::new();
  colors_tex.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  colors_tex.set_filter_mode(gli::NEAREST, gli::NEAREST);
  colors_tex.set_from_bytes(gli::RGBA, 64, 1, gli::RGBA, &palette::COLORS[0] as *const u8);

  scanline_screen.set_uniform(String::from("scanline"), scanline_tex.as_uniform_value());
  scanline_screen.set_uniform(String::from("colors"), colors_tex.as_uniform_value());
  // scanline program end

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
  nametable.set_wrap_mode(gli::REPEAT, gli::REPEAT);
  nametable.set_filter_mode(gli::NEAREST, gli::NEAREST);
  let nt_data: [u8; 64 * 60] = [0; 64 * 60];
  nametable.set_from_bytes(gli::R8UI, 64, 60, gli::RED_INTEGER, &nt_data[0] as *const u8);

  let mut attributes = gllite::texture::Texture::new();
  attributes.set_wrap_mode(gli::REPEAT, gli::REPEAT);
  attributes.set_filter_mode(gli::NEAREST, gli::NEAREST);
  let attr_data: [u8; 16 * 16] = [0; 16 * 16];
  attributes.set_from_bytes(gli::R8UI, 16, 16, gli::RED_INTEGER, &attr_data[0] as *const u8);

  // hack, need to make gl texture unit public
  let mut nt_unit = 0;
  if let UniformValue::Texture2D(u) = nametable.as_uniform_value() {
    nt_unit = u;
  }
  let mut attr_unit = 0;
  if let UniformValue::Texture2D(u) = attributes.as_uniform_value() {
    attr_unit = u;
  }

  let mut a_press = false;
  

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

  for mesh in sprite_meshes.iter_mut() {
    mesh.set_uniform(String::from("colors"), colors.as_uniform_value());
  }

  unsafe {
    gl::Enable(gl::BLEND);
    gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ZERO);
  }

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

    /*if delta < 16 {
      let diff = 16 - delta;
      let sleeptime = time::Duration::from_millis(diff as u64);
      thread::sleep(sleeptime);
      delta += diff;
    }*/
    if delta > 32 {
      delta = 32;
    }

    shell.update();
    if shell.should_exit() {
      break;
    }

    if shell.in_foreground() {
      for key in shell.keys_down.iter() {
        match key {
          VirtualKeyCode::Return => vm.mem.controller_0.start = true,
          VirtualKeyCode::RShift => vm.mem.controller_0.select = true,
          VirtualKeyCode::X => vm.mem.controller_0.a = true,
          VirtualKeyCode::Z => vm.mem.controller_0.b = true,
          VirtualKeyCode::Left => vm.mem.controller_0.left = true,
          VirtualKeyCode::Right => vm.mem.controller_0.right = true,
          VirtualKeyCode::Up => vm.mem.controller_0.up = true,
          VirtualKeyCode::Down => vm.mem.controller_0.down = true,
          VirtualKeyCode::A => {
            if !a_press {
              vm.mem.apu.test_note();
              a_press = true;
            }
          },
          _ => (),
        }
      }
      for key in shell.keys_up.iter() {
        match key {
          VirtualKeyCode::Return => vm.mem.controller_0.start = false,
          VirtualKeyCode::RShift => vm.mem.controller_0.select = false,
          VirtualKeyCode::X => vm.mem.controller_0.a = false,
          VirtualKeyCode::Z => vm.mem.controller_0.b = false,
          VirtualKeyCode::Left => vm.mem.controller_0.left = false,
          VirtualKeyCode::Right => vm.mem.controller_0.right = false,
          VirtualKeyCode::Up => vm.mem.controller_0.up = false,
          VirtualKeyCode::Down => vm.mem.controller_0.down = false,
          VirtualKeyCode::A => a_press = false,
          _ => (),
        }
      }

      let mut copied_scanline = false;
      // Run VM, draw result
      let cycles_for_this_frame = (delta * 1790) as u32;
      let mut total: u32 = 0;
      while total < cycles_for_this_frame {
        let mut cycles = vm.step() as u32;
        if vm.mem.dma_requested() {
          vm.mem.dma_copy();
          cycles += 514;
        }

        for _ in 0..(cycles * 3) {
          vm.mem.increment_clock();
          if !copied_scanline && vm.mem.ppu2.in_vblank() {
            copied_scanline = true;
            scanline_tex.set_from_bytes(gli::R8UI, 256, 240, gli::RED_INTEGER, vm.mem.ppu2.buffer_ptr());
          }
        }
        if vm.mem.ppu2.should_interrupt() {
          vm.cpu.nonmaskable_interrupt(&mut vm.mem);
        }
        total += cycles;
      }

      unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
      }

      scanline_prog.make_current();
      scanline_screen.draw();

      shell.swap_buffers();
    }
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
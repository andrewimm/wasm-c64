#![feature(box_syntax)]

use gllite;
use gllite::gli;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::time::{self, SystemTime};

use emushell::EmuShell;
use nesmemmap::mapper;

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
  //let mut pattern_1 = gllite::texture::Texture::new();
  //pattern_1.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  //pattern_1.set_filter_mode(gli::NEAREST, gli::NEAREST);
  let mut pattern_0_mem: Box<[u8; 0x1000]> = box [0; 0x1000];
  for i in 0..0x1000 {
    pattern_0_mem[i] = 0;
  }
  pattern_0_mem[0] = 0;
  pattern_0_mem[1] = 0xe0;
  pattern_0_mem[2] = 0xfc;
  pattern_0_mem[3] = 0x20;
  pattern_0_mem[4] = 0x20;
  pattern_0_mem[5] = 0x10;
  pattern_0_mem[6] = 0x3c;
  pattern_0_mem[7] = 0;
  pattern_0_mem[8] = 0;
  pattern_0_mem[9] = 0xe0;
  pattern_0_mem[10] = 0xfc;
  pattern_0_mem[11] = 0xd0;
  pattern_0_mem[12] = 0xdc;
  pattern_0_mem[13] = 0xee;
  pattern_0_mem[14] = 0xc0;
  pattern_0_mem[15] = 0xf8;
  pattern_0.set_from_bytes(gli::R8UI, 16, 256, gli::RED_INTEGER, &pattern_0_mem[0] as *const u8);

  screen.set_uniform(String::from("pattern_0"), pattern_0.as_uniform_value());

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
      unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
      }
      screen.draw();
    }

    shell.swap_buffers();
  }
}

fn load_mapper() -> Box<impl mapper::Mapper> {
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

  match mapper::create_mapper(&buffer) {
    Err(msg) => panic!("Failed to initialize Mapper: {}", msg),
    Ok(mapbox) => mapbox,
  }
}
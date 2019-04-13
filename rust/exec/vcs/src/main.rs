use gllite;
use gllite::gli;
use gllite::uniforms::UniformValue;
use glutin::{VirtualKeyCode};
use std::env;
use std::error::Error;
use std::ffi::c_void;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;
use std::thread;
use std::time::{self, SystemTime};

mod palette;
use mos6510::memory::Memory;
use vcsmemmap::tia::{ExecState, ScanlineState};
mod vm;
use vm::VM;

fn main() {
  let rom_data = load_rom_data();

  let mut shell = emushell::EmuShell::with_size_and_scale(256, 192, 2);
  shell.make_active_gl_context();
  shell.set_title("Rust 2600");

  let shader_frag = include_str!("shaders/frag.glsl");
  let shader_vert = include_str!("shaders/vert.glsl");

  let mut program = gllite::program::Program::new();
  program
    .add_shader(shader_vert, gl::VERTEX_SHADER)
    .add_shader(shader_frag, gl::FRAGMENT_SHADER)
    .compile();

  program.make_current();
  
  let prog = Rc::new(program);

  let mut screen = gllite::node::Node::for_program(Rc::clone(&prog));
  let vertices: [f32; 12] = [
    0.0, 0.0, 0.0, 1.0, 1.0, 0.0,
    1.0, 0.0, 0.0, 1.0, 1.0, 1.0,
  ];
  screen.add_attribute(String::from("a_position"));
  screen.buffer_data(&vertices);

  let mut palette_tex = gllite::texture::Texture::new();
  palette_tex.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  palette_tex.set_filter_mode(gli::NEAREST, gli::NEAREST);
  palette_tex.set_from_bytes(gli::RGBA, 128, 1, gli::RGBA, &palette::COLORS[0] as *const u8);

  screen.set_uniform(String::from("palette"), palette_tex.as_uniform_value());

  let mut screen_tex = gllite::texture::Texture::new();
  screen_tex.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  screen_tex.set_filter_mode(gli::NEAREST, gli::NEAREST);

  screen.set_uniform(String::from("screen"), screen_tex.as_uniform_value());

  let mut screen_tex_buffer: Box<[u8; 160 * 192]> = Box::new([0; 160 * 192]);

  let mut vm = VM::new();
  vm.mem.load_rom(rom_data.into_boxed_slice());
  vm.reset();

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
      for key in shell.keys_down.iter() {
        match key {
          VirtualKeyCode::Left => {
            vm.mem.riot.joystick_0_left = true;
            vm.mem.riot.joystick_0_right = false;
          },
          VirtualKeyCode::Right => {
            vm.mem.riot.joystick_0_right = true;
            vm.mem.riot.joystick_0_left = false;
          },
          VirtualKeyCode::Up => {
            vm.mem.riot.joystick_0_up = true;
            vm.mem.riot.joystick_0_down = true;
          },
          VirtualKeyCode::Down => {
            vm.mem.riot.joystick_0_down = true;
            vm.mem.riot.joystick_0_up = false;
          },
          _ => (),
        }
      }
      for key in shell.keys_up.iter() {
        match key {
          VirtualKeyCode::Left => {
            vm.mem.riot.joystick_0_left = false;
          },
          VirtualKeyCode::Right => {
            vm.mem.riot.joystick_0_right = false;
          },
          VirtualKeyCode::Up => {
            vm.mem.riot.joystick_0_up = false;
          },
          VirtualKeyCode::Down => {
            vm.mem.riot.joystick_0_down = false;
          },
          _ => (),
        }
      }

      while let ScanlineState::VSync = vm.mem.tia.get_scanline_state() {
        let cycles = match vm.mem.tia.get_exec_state() {
          ExecState::Run => vm.step() * 3,
          ExecState::Block => 1,
        };
        vm.mem.tia.increment_clock(cycles);
        for _ in 0..cycles {
          vm.mem.riot.increment_clock();
        }
      }
      let mut vsync = false;
      while !vsync {
        let cycles = match vm.mem.tia.get_exec_state() {
          ExecState::Run => vm.step() * 3,
          ExecState::Block => 1,
        };
        for _ in 0..cycles {
          vm.mem.tia.increment_clock(1);
          vm.mem.riot.increment_clock();

          match vm.mem.tia.get_scanline_state() {
            ScanlineState::Pixel(x, y, color) => {
              let addr = (y as u16 * 160) + (x as u16);
              screen_tex_buffer[addr as usize] = color;
            },
            ScanlineState::VSync => vsync = true,
            _ => (),
          }
        }
      }

      unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
      }

      screen_tex.set_from_bytes(gli::R8UI, 160, 192, gli::RED_INTEGER, &screen_tex_buffer[0] as *const u8);
      screen.draw();

      shell.swap_buffers();
    }
  }
}

fn load_rom_data() -> Vec<u8> {
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
  buffer
}
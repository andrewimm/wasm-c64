use glutin::{VirtualKeyCode};
use gllite::gli;
use gllite::uniforms::UniformValue;
use std::rc::Rc;
use std::cmp;
use std::thread;
use std::time::{self, SystemTime};

mod vm;
use vm::VM;

fn main() {
  let mut shell = emushell::EmuShell::with_size_and_scale(384, 272, 2);
  shell.make_active_gl_context();

  let mut last_frames: [u8;32] = [0;32];
  let mut last_frames_pointer = 0;

  // include shaders
  let shader_textmode_frag = include_str!("shaders/textmode_frag.glsl");
  let shader_textmode_vert = include_str!("shaders/textmode_vert.glsl");

  let mut textmode_program = gllite::program::Program::new();
  textmode_program
    .add_shader(shader_textmode_vert, gl::VERTEX_SHADER)
    .add_shader(shader_textmode_frag, gl::FRAGMENT_SHADER)
    .compile();

  textmode_program.make_current();
  
  let p = Rc::new(textmode_program);

  let mut screen = gllite::node::Node::for_program(Rc::clone(&p));
  let vertices: [f32; 12] = [
    0.0, 0.0, 0.0, 1.0, 1.0, 0.0,
    1.0, 0.0, 0.0, 1.0, 1.0, 1.0,
  ];
  screen.add_attribute(String::from("a_position"));
  screen.buffer_data(&vertices);
  
  let color_tex = gllite::texture::Texture::new();
  color_tex.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  color_tex.set_filter_mode(gli::NEAREST, gli::NEAREST);
  color_tex.set_from_bytes(gli::RGBA, 1, 16, gli::RGBA, &COLORS[0] as *const u8);
  let mut screen_mem_tex = gllite::texture::Texture::new();
  screen_mem_tex.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  screen_mem_tex.set_filter_mode(gli::NEAREST, gli::NEAREST);
  let mut char_mem_tex = gllite::texture::Texture::new();
  char_mem_tex.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  char_mem_tex.set_filter_mode(gli::NEAREST, gli::NEAREST);
  let mut color_mem_tex = gllite::texture::Texture::new();
  color_mem_tex.set_wrap_mode(gli::CLAMP_TO_EDGE, gli::CLAMP_TO_EDGE);
  color_mem_tex.set_filter_mode(gli::NEAREST, gli::NEAREST);

  let mut vm = VM::new();

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

    let fps = 1000 / cmp::max(delta, 1);
    last_frames[last_frames_pointer as usize] = fps as u8;
    let mut sum: u32 = 0;
    for i in 0..32 {
      sum += last_frames[((last_frames_pointer + i) & 31) as usize] as u32;
    }
    last_frames_pointer = (last_frames_pointer + 1) & 31;
    if last_frames_pointer == 0 {
      let title = format!("Rust C64 - {}fps", sum / 32);
      shell.set_title(&title[0..title.len()]);
    }

    shell.update();
    if shell.should_exit() {
      break;
    }

    if shell.in_foreground() {
      for key in shell.keys_down.iter() {
        let code = derive_keycode(key);
        if code != 255 {
          vm.mem.cia.keydown(code);
        }
      }
      for key in shell.keys_up.iter() {
        let code = derive_keycode(key);
        if code != 255 {
          vm.mem.cia.keyup(code);
        }
      }

      // run vm for delta ms
      vm.run_for_ms(delta as u32);

      // load char mem
      load_char_mem(&mut vm, &mut char_mem_tex);
      // load screen mem
      load_screen_mem(&mut vm, &mut screen_mem_tex);
      // load color mem
      load_color_mem(&mut vm, &mut color_mem_tex);
      // set mode

      screen.set_uniform(String::from("u_bgcolor"), UniformValue::Int(vm.mem.vic.background_color as i32));
      screen.set_uniform(String::from("u_framecolor"), UniformValue::Int(vm.mem.vic.border_color as i32));
      screen.set_uniform(String::from("u_colors"), color_tex.as_uniform_value());
      screen.set_uniform(String::from("u_charmem"), char_mem_tex.as_uniform_value());
      screen.set_uniform(String::from("u_screenmem"), screen_mem_tex.as_uniform_value());
      screen.set_uniform(String::from("u_colormem"), color_mem_tex.as_uniform_value());

      unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
      }
      screen.draw();
    }

    shell.swap_buffers();
  }
}

const COLORS: [u8; 4 * 16] = [
  0x00, 0x00, 0x00, 0x00,
  0xff, 0xff, 0xff, 0x00,
  0x68, 0x37, 0x2b, 0x00,
  0x70, 0xa4, 0xb2, 0x00,
  0x6f, 0x3d, 0x86, 0x00,
  0x58, 0x8d, 0x43, 0x00,
  0x35, 0x28, 0x79, 0x00,
  0xb8, 0xc7, 0x6f, 0x00,
  0x6f, 0x4f, 0x25, 0x00,
  0x43, 0x39, 0x00, 0x00,
  0x9a, 0x67, 0x59, 0x00,
  0x44, 0x44, 0x44, 0x00,
  0x6c, 0x6c, 0x6c, 0x00,
  0x9a, 0xd2, 0x84, 0x00,
  0x6c, 0x5e, 0xb5, 0x00,
  0x95, 0x95, 0x95, 0x00,
];

fn load_char_mem(vm: &mut VM, tex: &gllite::texture::Texture) {
  tex.set_from_bytes(gl::R8UI, 8, 256, gl::RED_INTEGER, vm.mem.ram_rom.char_ptr())
}

fn load_screen_mem(vm: &mut VM, tex: &gllite::texture::Texture) {
  tex.set_from_bytes(gl::R8UI, 40, 25, gl::RED_INTEGER, vm.mem.ram_rom.screen_ptr())
}

fn load_color_mem(vm: &mut VM, tex: &gllite::texture::Texture) {
  tex.set_from_bytes(gl::R8UI, 1024, 1, gl::RED_INTEGER, vm.mem.ram_rom.color_ptr())
}

fn derive_keycode(code: &VirtualKeyCode) -> u8 {
  match code {
    VirtualKeyCode::Escape => 63,
    VirtualKeyCode::Grave => 57,
    VirtualKeyCode::Key1 => 56,
    VirtualKeyCode::Key2 => 59,
    VirtualKeyCode::Key3 => 8,
    VirtualKeyCode::Key4 => 11,
    VirtualKeyCode::Key5 => 16,
    VirtualKeyCode::Key6 => 19,
    VirtualKeyCode::Key7 => 24,
    VirtualKeyCode::Key8 => 27,
    VirtualKeyCode::Key9 => 32,
    VirtualKeyCode::Key0 => 35,
    VirtualKeyCode::Minus => 40,
    VirtualKeyCode::Equals => 53,
    VirtualKeyCode::Back => 0,
    VirtualKeyCode::Tab => 58,
    VirtualKeyCode::Q => 62,
    VirtualKeyCode::W => 9,
    VirtualKeyCode::E => 14,
    VirtualKeyCode::R => 17,
    VirtualKeyCode::T => 22,
    VirtualKeyCode::Y => 25,
    VirtualKeyCode::U => 30,
    VirtualKeyCode::I => 33,
    VirtualKeyCode::O => 38,
    VirtualKeyCode::P => 41,
    VirtualKeyCode::LBracket => 46,
    VirtualKeyCode::RBracket => 49,
    VirtualKeyCode::Return => 1,
    VirtualKeyCode::Capital => 52,
    VirtualKeyCode::A => 10,
    VirtualKeyCode::S => 13,
    VirtualKeyCode::D => 18,
    VirtualKeyCode::F => 21,
    VirtualKeyCode::G => 26,
    VirtualKeyCode::H => 29,
    VirtualKeyCode::J => 34,
    VirtualKeyCode::K => 37,
    VirtualKeyCode::L => 42,
    VirtualKeyCode::Semicolon => 45,
    VirtualKeyCode::Apostrophe => 50,
    VirtualKeyCode::LShift => 15,
    VirtualKeyCode::Z => 12,
    VirtualKeyCode::X => 23,
    VirtualKeyCode::C => 20,
    VirtualKeyCode::V => 31,
    VirtualKeyCode::B => 28,
    VirtualKeyCode::N => 39,
    VirtualKeyCode::M => 36,
    VirtualKeyCode::Comma => 47,
    VirtualKeyCode::Period => 44,
    VirtualKeyCode::Slash => 55,
    VirtualKeyCode::LControl => 61,
    VirtualKeyCode::Space => 60,
    _ => 255,
  }
}

use glutin::{ContextBuilder, ElementState, Event, EventsLoop, VirtualKeyCode, WindowedContext, WindowBuilder, WindowEvent};
use glutin::dpi::{LogicalSize};
use glutin::ContextTrait;
use gl;
use gl::types::{GLint, GLfloat, GLsizeiptr};
use std::cmp;
use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::thread;
use std::time::{self, SystemTime};

mod vm;
use vm::VM;

struct EmuShell {
  events_loop: EventsLoop,
  context: WindowedContext,
  width: u32,
  height: u32,
  scale: u32,

  close_requested: bool,
  foregrounded: bool,

  pub keys_down: Vec<u8>,
  pub keys_up: Vec<u8>,

  last_frames: [u8;32],
  last_frames_pointer: u8,
}

impl EmuShell {
  pub fn new() -> EmuShell {
    let width = 384;
    let height = 272;
    let scale = 2;
    let mut events_loop = EventsLoop::new();
    let wb = WindowBuilder::new()
      .with_title("EmuShell")
      .with_resizable(false)
      .with_dimensions(LogicalSize::from((width * scale, height * scale)));
    let context = ContextBuilder::new()
      .with_vsync(true)
      .build_windowed(wb, &events_loop)
      .unwrap();
    
    unsafe {
      context.make_current().unwrap();
      gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
      gl::ClearColor(1.0, 0.0, 1.0, 1.0);
    }

    EmuShell {
      events_loop: events_loop,
      context: context,
      width: width,
      height: height,
      scale: 1,

      close_requested: false,
      foregrounded: false,
      keys_down: Vec::new(),
      keys_up: Vec::new(),
      last_frames: [0;32],
      last_frames_pointer: 0,
    }
  }

  pub fn set_title(&mut self, title: &str) {
    self.context.set_title(title);
  }

  pub fn set_scale(&mut self, scale: u32) {
    self.scale = scale;
    self.resize();
  }

  fn resize(&mut self) {
    let size = LogicalSize::from((self.width * self.scale, self.height * self.scale));
    self.context.set_inner_size(size);
  }

  pub fn should_exit(&self) -> bool {
    self.close_requested
  }

  pub fn in_foreground(&self) -> bool {
    self.foregrounded
  }

  pub fn update(&mut self) {
    let mut close_requested = false;
    let mut background = false;
    let mut foregrounded = self.foregrounded;
    let keys_down = &mut self.keys_down;
    let keys_up = &mut self.keys_up;
    self.events_loop.poll_events(|event| {
      match event {
        Event::WindowEvent {event, ..} => match event {
          WindowEvent::CloseRequested => close_requested = true,
          WindowEvent::Focused(flag) => foregrounded = flag,
          WindowEvent::KeyboardInput {input, ..} => {
            let code = derive_keycode(input.virtual_keycode);
            if code != 255 {
              if input.state == ElementState::Pressed {
                keys_down.push(code);
              } else {
                keys_up.push(code);
              }
            }
          },
          _ => (),
        },
        _ => (),
      }
    });
    self.close_requested = close_requested;
    self.foregrounded = foregrounded;
  }

  pub fn swap_buffers(&mut self) {
    self.context.swap_buffers().unwrap();
  }
}

struct TextModeUniforms {
  bg_color: i32,
  frame_color: i32,
  charmem: i32,
  colors: i32,
  screenmem: i32,
  colormem: i32,
}

fn main() {
  let mut shell = EmuShell::new();

  // include shaders
  let shader_textmode_frag = include_str!("shaders/textmode_frag.glsl");
  let shader_textmode_vert = include_str!("shaders/textmode_vert.glsl");

  let textmode_program = create_program(
    create_shader(gl::VERTEX_SHADER, shader_textmode_vert),
    create_shader(gl::FRAGMENT_SHADER, shader_textmode_frag)
  );
  let screen_vao = get_screen_vao();
  let color_tex = get_color_texture();
  let screen_mem_tex = get_screen_mem_texture();
  let char_mem_tex = get_char_mem_texture();
  let color_mem_tex = get_color_mem_texture();
  let uniforms = get_uniforms(textmode_program);

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
    let last_frames_pointer = shell.last_frames_pointer;
    shell.last_frames[last_frames_pointer as usize] = fps as u8;
    let mut sum: u32 = 0;
    for i in 0..32 {
      sum += shell.last_frames[((last_frames_pointer + i) & 31) as usize] as u32;
    }
    shell.last_frames_pointer = (last_frames_pointer + 1) & 31;
    if last_frames_pointer == 0 {
      let title = format!("Rust C64 - {}fps", sum / 32);
      shell.set_title(&title[0..title.len()]);
    }

    shell.update();
    if shell.should_exit() {
      break;
    }

    if shell.in_foreground() {
      while shell.keys_down.len() > 0 {
        let code = shell.keys_down.pop();
        match code {
          Some(c) => vm.mem.cia.keydown(c),
          _ => (),
        };
      }
      while shell.keys_up.len() > 0 {
        let code = shell.keys_up.pop();
        match code {
          Some(c) => vm.mem.cia.keyup(c),
          _ => (),
        }
      }

      // run vm for delta ms
      vm.run_for_ms(delta as u32);

      // load char mem
      load_char_mem(&mut vm, char_mem_tex);
      // load screen mem
      load_screen_mem(&mut vm, screen_mem_tex);
      // load color mem
      load_color_mem(&mut vm, color_mem_tex);
      // set mode

      unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::UseProgram(textmode_program);
        gl::BindVertexArray(screen_vao);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, color_tex);
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_2D, char_mem_tex);
        gl::ActiveTexture(gl::TEXTURE2);
        gl::BindTexture(gl::TEXTURE_2D, screen_mem_tex);
        gl::ActiveTexture(gl::TEXTURE3);
        gl::BindTexture(gl::TEXTURE_2D, color_mem_tex);

        gl::Uniform1i(uniforms.bg_color, vm.mem.vic.background_color as i32);
        gl::Uniform1i(uniforms.frame_color, vm.mem.vic.border_color as i32);
        gl::Uniform1i(uniforms.colors, 0);
        gl::Uniform1i(uniforms.charmem, 1);
        gl::Uniform1i(uniforms.screenmem, 2);
        gl::Uniform1i(uniforms.colormem, 3);
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
      }
    }

    shell.swap_buffers();
  }
}

fn create_shader(shader_type: u32, source: &str) -> u32 {
  unsafe {
    let shader = gl::CreateShader(shader_type);
    let c_str = CString::new(source.as_bytes()).unwrap();
    gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    let mut success = gl::FALSE as GLint;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success != gl::TRUE as GLint {
      let mut bytes: [i8; 512] = [0;512];
      gl::GetShaderInfoLog(shader, 512, ptr::null_mut(), &mut bytes[0] as *mut i8);
      let u8bytes = unsafe { &*(&bytes[..] as *const [i8] as *const [u8]) };
      println!("Shader Error: {}", std::str::from_utf8(u8bytes).unwrap());
      gl::DeleteShader(shader);
      panic!("Failed to compile shader");
    }
    shader
  }
}

fn create_program(vertex_shader: u32, fragment_shader: u32) -> u32 {
  unsafe {
    let program = gl::CreateProgram();
    gl::AttachShader(program, vertex_shader);
    gl::AttachShader(program, fragment_shader);
    gl::LinkProgram(program);

    let mut success = gl::FALSE as GLint;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    if success != gl::TRUE as GLint {
      gl::DeleteProgram(program);
      panic!("Failed to link program");
    }
    program
  }
}

fn get_screen_vao() -> u32 {
  unsafe {
    let vertices: [f32; 12] = [
      0.0, 0.0, 0.0, 1.0, 1.0, 0.0,
      1.0, 0.0, 0.0, 1.0, 1.0, 1.0,
    ];
    let mut vao = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);
    let mut vbo = 0;
    gl::GenBuffers(1, &mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    let float_size = mem::size_of::<GLfloat>();
    gl::BufferData(
      gl::ARRAY_BUFFER,
      (vertices.len() * float_size) as GLsizeiptr,
      &vertices[0] as *const f32 as *const c_void,
      gl::STATIC_DRAW
    );
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
    vao
  }
}

fn get_uniforms(program: u32) -> TextModeUniforms {
  unsafe {
    let bg_color_location = gl::GetUniformLocation(
      program,
      CString::new("u_bgcolor").unwrap().as_ptr()
    );
    let frame_color_location = gl::GetUniformLocation(
      program,
      CString::new("u_framecolor").unwrap().as_ptr()
    );
    let charmem_location = gl::GetUniformLocation(
      program,
      CString::new("u_charmem").unwrap().as_ptr()
    );
    let colors_location = gl::GetUniformLocation(
      program,
      CString::new("u_colors").unwrap().as_ptr()
    );
    let screenmem_location = gl::GetUniformLocation(
      program,
      CString::new("u_screenmem").unwrap().as_ptr()
    );
    let colormem_location = gl::GetUniformLocation(
      program,
      CString::new("u_colormem").unwrap().as_ptr()
    );
    TextModeUniforms {
      bg_color: bg_color_location,
      frame_color: frame_color_location,
      charmem: charmem_location,
      colors: colors_location,
      screenmem: screenmem_location,
      colormem: colormem_location,
    }
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

fn get_color_texture() -> u32 {
  unsafe {
    let mut texture = 0;
    gl::GenTextures(1, &mut texture);
    gl::BindTexture(gl::TEXTURE_2D, texture);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, 1, 16, 0, gl::RGBA, gl::UNSIGNED_BYTE, &COLORS[0] as *const u8 as *const c_void);
    texture
  }
}

fn get_char_mem_texture() -> u32 {
  unsafe {
    let mut texture = 0;
    gl::GenTextures(1, &mut texture);
    gl::BindTexture(gl::TEXTURE_2D, texture);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    texture
  }
}

fn get_screen_mem_texture() -> u32 {
  unsafe {
    let mut texture = 0;
    gl::GenTextures(1, &mut texture);
    gl::BindTexture(gl::TEXTURE_2D, texture);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    texture
  }
}

fn get_color_mem_texture() -> u32 {
  unsafe {
    let mut texture = 0;
    gl::GenTextures(1, &mut texture);
    gl::BindTexture(gl::TEXTURE_2D, texture);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    texture
  }
}

fn load_char_mem(vm: &mut VM, tex: u32) {
  unsafe {
    gl::BindTexture(gl::TEXTURE_2D, tex);
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8UI as i32, 8, 256, 0, gl::RED_INTEGER, gl::UNSIGNED_BYTE, vm.mem.ram_rom.char_ptr() as *const c_void);
  }
}

fn load_screen_mem(vm: &mut VM, tex: u32) {
  unsafe {
    gl::BindTexture(gl::TEXTURE_2D, tex);
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8UI as i32, 40, 25, 0, gl::RED_INTEGER, gl::UNSIGNED_BYTE, vm.mem.ram_rom.screen_ptr() as *const c_void);
  }
}

fn load_color_mem(vm: &mut VM, tex: u32) {
  unsafe {
    gl::BindTexture(gl::TEXTURE_2D, tex);
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8UI as i32, 1024, 1, 0, gl::RED_INTEGER, gl::UNSIGNED_BYTE, vm.mem.ram_rom.color_ptr() as *const c_void);
  }
}

fn derive_keycode(code: Option<VirtualKeyCode>) -> u8 {
  match code {
    Some(c) => match c {
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
    },
    _ => 255,
  }
}

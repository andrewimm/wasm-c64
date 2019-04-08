use glutin::{ContextBuilder, ElementState, Event, EventsLoop, VirtualKeyCode, WindowedContext, WindowBuilder, WindowEvent};
use glutin::dpi::{LogicalSize};
use glutin::ContextTrait;
use std::collections::HashSet;

pub struct EmuShell {
  events_loop: EventsLoop,
  context: WindowedContext,
  width: u32,
  height: u32,
  scale: u32,

  close_requested: bool,
  foregrounded: bool,

  pub keys_down: HashSet<VirtualKeyCode>,
  pub keys_up: HashSet<VirtualKeyCode>,
}

impl EmuShell {
  pub fn new() -> EmuShell {
    EmuShell::with_size_and_scale(400, 400, 1)
  }

  pub fn with_size_and_scale(width: u32, height: u32, scale: u32) -> EmuShell {
    let events_loop = EventsLoop::new();
    let wb = WindowBuilder::new()
      .with_title("EmuShell")
      .with_resizable(false)
      .with_dimensions(LogicalSize::from((width * scale, height * scale)));
    let context = ContextBuilder::new()
      .with_vsync(true)
      .build_windowed(wb, &events_loop)
      .unwrap();
    
    EmuShell {
      events_loop: events_loop,
      context: context,
      width: width,
      height: height,
      scale: scale,

      close_requested: false,
      foregrounded: false,
      keys_down: HashSet::new(),
      keys_up: HashSet::new(),
    }
  }

  pub fn make_active_gl_context(&self) {
    unsafe {
      self.context.make_current().unwrap();
      gl::load_with(|symbol| self.context.get_proc_address(symbol) as *const _);
    }
  }

  pub fn set_title(&mut self, title: &str) {
    self.context.set_title(title);
  }

  pub fn set_scale(&mut self, scale: u32) {
    self.scale = scale;
    self.resize();
  }

  pub fn set_size(&mut self, width: u32, height: u32) {
    self.width = width;
    self.height = height;
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
    let mut foregrounded = self.foregrounded;
    let keys_down = &mut self.keys_down;
    let keys_up = &mut self.keys_up;
    self.events_loop.poll_events(|event| {
      match event {
        Event::WindowEvent {event, ..} => match event {
          WindowEvent::CloseRequested => close_requested = true,
          WindowEvent::Focused(flag) => foregrounded = flag,
          WindowEvent::KeyboardInput {input, ..} => {
            if let Some(code) = input.virtual_keycode {
              if input.state == ElementState::Pressed {
                keys_up.remove(&code);
                keys_down.insert(code);
              } else {
                keys_down.remove(&code);
                keys_up.insert(code);
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
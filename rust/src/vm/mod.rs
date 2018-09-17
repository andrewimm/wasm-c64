pub mod cpu;
pub mod mem;
pub mod memmap;

const CYCLES_PER_MS: u32 = 1023;

pub struct VM {
  pub cpu: cpu::CPU,
  pub mem: memmap::MemMap,
}

impl VM {
pub fn step(&mut self) -> u8 {
  return self.cpu.step(&mut self.mem);
}

pub fn run_ms(&mut self, ms: u32) {
  let cycles = CYCLES_PER_MS * ms;
  let mut ran = 0;
  while ran < cycles {
    let step_time = self.step();
    ran += step_time;
  }
}
}


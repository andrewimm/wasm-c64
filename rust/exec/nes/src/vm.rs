use mos6510::cpu::CPU;
use nesmemmap::mapper;
use nesmemmap::memmap::MemMap;

pub struct VM {
  pub cpu: CPU,
  pub mem: MemMap,
}

const CYCLES_PER_MS: u32 = 1023;

impl VM {
  pub fn new(mapper: Box<mapper::Mapper>) -> VM {
    let mut vm = VM {
      cpu: CPU::new(),
      mem: MemMap::new(mapper),
    };

    vm.reset();
    return vm;
  }

  pub fn step(&mut self) -> u8 {
    return self.cpu.step(&mut self.mem);
  }

  pub fn run_for_cycles(&mut self, cycles: u32) {
    let mut ran = 0;
    while ran < cycles {
      let step_time = self.step();
      // TODO: update timers
      ran += step_time as u32;
    }
  }

  pub fn reset(&mut self) {
    self.cpu.reset(&mut self.mem);
  }
}
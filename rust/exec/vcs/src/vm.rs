use mos6510::cpu::CPU;
use vcsmemmap::memmap::MemMap;

pub struct VM {
  pub cpu: CPU,
  pub mem: MemMap,
}

impl VM {
  pub fn new() -> VM {
    let mut vm = VM {
      cpu: CPU::new(),
      mem: MemMap::new(),
    };

    vm.reset();
    return vm;
  }

  pub fn step(&mut self) -> u8 {
    return self.cpu.step(&mut self.mem);
  }

  pub fn reset(&mut self) {
    self.cpu.reset(&mut self.mem);
  }
}
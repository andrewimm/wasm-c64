extern crate mos6510;
extern crate c64memmap;

use self::mos6510::cpu::CPU;
use self::c64memmap::memmap::MemMap;

const CYCLES_PER_MS: u32 = 1023;

pub struct VM {
  pub cpu: CPU,
  pub mem: MemMap,
}

impl VM {
pub fn new() -> VM {
  VM {
    cpu: CPU::new(),
    mem: MemMap::new(),
  }
}

pub fn step(&mut self) -> u8 {
  return self.cpu.step(&mut self.mem);
}

pub fn run_ms(&mut self, ms: u32) {
  let cycles = CYCLES_PER_MS * ms;
  let mut ran = 0;
  while ran < cycles {
    let step_time = self.step();
    if self.mem.cia.update_timers(step_time) {
      // cia interrupt fired
      self.cpu.interrupt_request(&mut self.mem);
    }
    ran += step_time as u32;
  }
}

pub fn reset(&mut self) {
  self.cpu.reset(&mut self.mem);
}
}

#[cfg(test)]
mod tests {
  use vm::VM;
  use vm::mos6510::memory::Memory;

  #[test]
  fn basic_ops() {
    let mut vm = VM::new();
    vm.mem.set_basic_rom(vec![
      0xa9, 0x22, // LDA #$22
      0x69, 0x11, // ADC #$11
    ], 0);
    vm.cpu.pc = 0xa000;
    vm.cpu.step(&mut vm.mem);
    assert_eq!(vm.cpu.acc, 0x22);
    vm.cpu.step(&mut vm.mem);
    assert_eq!(vm.cpu.acc, 0x33);
  }

  #[test]
  fn memory_ops() {
    let mut vm = VM::new();
    vm.mem.set_basic_rom(vec![
      0xa9, 0x40, // LDA #$40
      0x8d, 0x05, 0x20, // STA #$2005
      0xac, 0x05, 0x20, // LDY #$2005
    ], 0);
    vm.cpu.pc = 0xa000;
    vm.cpu.step(&mut vm.mem);
    vm.cpu.step(&mut vm.mem);
    assert_eq!(vm.mem.get_byte(0x2005), 0x40);
    vm.cpu.step(&mut vm.mem);
    assert_eq!(vm.cpu.y, 0x40);
  }
}

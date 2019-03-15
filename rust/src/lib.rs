pub mod vm;

use std::mem;
use vm::VM;

#[no_mangle]
pub fn create_vm() -> *mut VM {
  let vm = VM::new();
  let b = Box::new(vm);
  return Box::into_raw(b);
}

#[no_mangle]
pub fn reset(raw: *mut VM) {
  unsafe {
    let mut vm = Box::from_raw(raw);
    vm.reset();
    mem::forget(vm);
  }
}

#[no_mangle]
pub fn get_char_pointer(raw: *mut VM) -> *mut u8 {
  unsafe {
    let mut vm = Box::from_raw(raw);
    let ptr = vm.mem.ram_rom.char_ptr();
    mem::forget(vm);
    return ptr;
  }
}

#[no_mangle]
pub fn get_kernal_pointer(raw: *mut VM) -> *mut u8 {
  unsafe {
    let mut vm = Box::from_raw(raw);
    let ptr = vm.mem.ram_rom.kernal_ptr();
    mem::forget(vm);
    return ptr;
  }
}

#[no_mangle]
pub fn get_basic_pointer(raw: *mut VM) -> *mut u8 {
  unsafe {
    let mut vm = Box::from_raw(raw);
    let ptr = vm.mem.ram_rom.basic_ptr();
    mem::forget(vm);
    return ptr;
  }
}

#[no_mangle]
pub fn get_ram_pointer(raw: *mut VM) -> *mut u8 {
  unsafe {
    let mut vm = Box::from_raw(raw);
    let ptr = vm.mem.ram_rom.ram_ptr();
    mem::forget(vm);
    return ptr;
  }
}

#[no_mangle]
pub fn get_color_pointer(raw: *mut VM) -> *mut u8 {
  unsafe {
    let mut vm = Box::from_raw(raw);
    let ptr = vm.mem.ram_rom.color_ptr();
    mem::forget(vm);
    return ptr;
  }
}

#[no_mangle]
pub fn step_vm(raw: *mut VM) -> u8 {
  unsafe {
    let mut vm = Box::from_raw(raw);
    let time = vm.step();
    mem::forget(vm);
    return time;
  }
}

#[no_mangle]
pub fn run_vm(raw: *mut VM, ms: u32) {
  unsafe {
    let mut vm = Box::from_raw(raw);
    vm.run_ms(ms);
    mem::forget(vm);
  }
}

#[no_mangle]
pub fn get_register(raw: *mut VM, register: u32) -> u16 {
  unsafe {
    let vm = Box::from_raw(raw);
    let value = match register {
      0 => vm.cpu.acc as u16,
      1 => vm.cpu.x as u16,
      2 => vm.cpu.y as u16,
      3 => vm.cpu.status as u16,
      4 => vm.cpu.stack as u16,
      5 => vm.cpu.pc,
      _ => 0,
    };
    mem::forget(vm);
    return value;
  }
}

#[no_mangle]
pub fn keydown(raw: *mut VM, key: u8) {
  unsafe {
    let mut vm = Box::from_raw(raw);
    vm.mem.cia.keydown(key);
    mem::forget(vm);
  }
}

#[no_mangle]
pub fn keyup(raw: *mut VM, key: u8) {
  unsafe {
    let mut vm = Box::from_raw(raw);
    vm.mem.cia.keyup(key);
    mem::forget(vm);
  }
}

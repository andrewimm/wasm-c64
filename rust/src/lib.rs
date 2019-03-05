#![feature(box_syntax)]

pub mod vm;

use std::mem;
use vm::VM;
use vm::cpu::Register;

#[no_mangle]
pub fn create_vm() -> *mut VM {
  let vm = VM {
    cpu: vm::cpu::create_cpu(),
    mem: vm::memmap::create_memmap(),
  };
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
    let ptr = vm.mem.mem.char_ptr();
    mem::forget(vm);
    return ptr;
  }
}

#[no_mangle]
pub fn get_kernal_pointer(raw: *mut VM) -> *mut u8 {
  unsafe {
    let mut vm = Box::from_raw(raw);
    let ptr = vm.mem.mem.kernal_ptr();
    mem::forget(vm);
    return ptr;
  }
}

#[no_mangle]
pub fn get_basic_pointer(raw: *mut VM) -> *mut u8 {
  unsafe {
    let mut vm = Box::from_raw(raw);
    let ptr = vm.mem.mem.basic_ptr();
    mem::forget(vm);
    return ptr;
  }
}

#[no_mangle]
pub fn get_ram_pointer(raw: *mut VM) -> *mut u8 {
  unsafe {
    let mut vm = Box::from_raw(raw);
    let ptr = vm.mem.mem.ram_ptr();
    mem::forget(vm);
    return ptr;
  }
}

#[no_mangle]
pub fn get_color_pointer(raw: *mut VM) -> *mut u8 {
  unsafe {
    let mut vm = Box::from_raw(raw);
    let ptr = vm.mem.mem.color_ptr();
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
      0 => vm.cpu.get_register(Register::Acc) as u16,
      1 => vm.cpu.get_register(Register::X) as u16,
      2 => vm.cpu.get_register(Register::Y) as u16,
      3 => vm.cpu.get_register(Register::Status) as u16,
      4 => vm.cpu.get_register(Register::Stack) as u16,
      5 => vm.cpu.get_pc(),
      _ => 0,
    };
    mem::forget(vm);
    return value;
  }
}
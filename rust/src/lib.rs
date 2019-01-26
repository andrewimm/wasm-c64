#![feature(box_syntax)]

pub mod vm;

use std::mem;
use vm::VM;

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
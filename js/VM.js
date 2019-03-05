(function() {
function fetchAndInstantiate(url, importObject) {
  return fetch(url).then(
    res => res.arrayBuffer()
  ).then(
    bytes => WebAssembly.instantiate(bytes, importObject)
  ).then(
    results => results.instance
  );
}

function memcpy(memory, source, dest) {
  for (let i = 0; i < source.length; i++) {
    memory[dest + i] = source[i];
  }
}

const PATH = 'build/wasm_c64.wasm';

function loadWASM(vm) {
  return fetchAndInstantiate(PATH, {
    env: {
      // methods exposed to wasm
    },
  }).then(instance => {
    return {
      memory: instance.exports.memory,
      createVM: instance.exports.create_vm,
      getCharPointer: instance.exports.get_char_pointer,
      getKernalPointer: instance.exports.get_kernal_pointer,
      getBasicPointer: instance.exports.get_basic_pointer,
      getColorPointer: instance.exports.get_color_pointer,
      getRAMPointer: instance.exports.get_ram_pointer,
      stepVM: instance.exports.step_vm,
      runVMFor: instance.exports.run_vm,
      reset: instance.exports.reset,
      getRegister: instance.exports.get_register,
    };
  });
}

function b64ToByteArray(str) {
  const binString = atob(str);
  const len = binString.length;
  const arr = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    arr[i] = binString.charCodeAt(i);
  }
  return arr;
}

class VM {
  constructor(gl) {
    this.graphics = new Graphics(gl);

    this._ready = loadWASM(this);
    this._ready.then(mod => {
      this.mod = mod;
      this.c64 = mod.createVM();
      const mem = {
        charPtr: mod.getCharPointer(this.c64),
        kernalPtr: mod.getKernalPointer(this.c64),
        basicPtr: mod.getBasicPointer(this.c64),
        colorPtr: mod.getColorPointer(this.c64),
        ramPtr: mod.getRAMPointer(this.c64),
      };
      const buffer = mod.memory.buffer;
      mem.char = new Uint8Array(buffer, mem.charPtr, 0x1000);
      mem.kernal = new Uint8Array(buffer, mem.kernalPtr, 0x2000);
      mem.basic = new Uint8Array(buffer, mem.basicPtr, 0x2000);
      mem.color = new Uint8Array(buffer, mem.colorPtr, 0x400);
      mem.screen = new Uint8Array(buffer, mem.ramPtr + 0x400, 1000);
      this.mem = mem;

      memcpy(this.mem.char, b64ToByteArray(ROM.CHAR), 0);
      memcpy(this.mem.kernal, b64ToByteArray(ROM.KERNAL), 0);
      memcpy(this.mem.basic, b64ToByteArray(ROM.BASIC), 0);

      mod.reset(this.c64);
      this.printRegisters();
    });

    this._lastFrame = 0;

    this.frame = this.frame.bind(this);
  }

  step() {
    this.mod.stepVM(this.c64);
  }

  frame(ms) {
    if (this._lastFrame === 0) {
      this._lastFrame = ms;
    }
    const delta = ms - this._lastFrame;
    this._lastFrame = ms;

    // collect input

    // update cpu state
    this.mod.runVMFor(this.c64, delta);

    // draw screen
    this.graphics.loadCharMem(this.mem.char);
    this.graphics.loadScreenMem(this.mem.screen);
    this.graphics.loadColorMem(this.mem.color);
    this.graphics.draw();

    requestAnimationFrame(this.frame);
  }

  ready() {
    return this._ready;
  }

  printRegisters() {
    console.table({
      Acc: this.mod.getRegister(this.c64, 0).toString(16),
      X: this.mod.getRegister(this.c64, 1).toString(16),
      Y: this.mod.getRegister(this.c64, 2).toString(16),
      Status: this.mod.getRegister(this.c64, 3).toString(2),
      PC: this.mod.getRegister(this.c64, 5).toString(16),
    });
  }
}

window.VM = VM;
})();
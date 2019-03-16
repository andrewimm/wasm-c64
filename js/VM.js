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
      console_log: msg => console.log(msg),
      console_log_b: n => console.log(n.toString(2)),
      console_error: err => console.error(err),
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
      keydown: instance.exports.keydown,
      keyup: instance.exports.keyup,
      getBorderColor: instance.exports.get_border_color,
      getBgColor: instance.exports.get_bg_color,
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

const KEYBOARD = {
  Escape: 63,
  Backquote: 57,
  Digit1: 56,
  Digit2: 59,
  Digit3: 8,
  Digit4: 11,
  Digit5: 16,
  Digit6: 19,
  Digit7: 24,
  Digit8: 27,
  Digit9: 32,
  Digit0: 35,
  Minus: 40,
  Equal: 53,
  Backspace: 0,
  Tab: 58,
  KeyQ: 62,
  KeyW: 9,
  KeyE: 14,
  KeyR: 17,
  KeyT: 22,
  KeyY: 25,
  KeyU: 30,
  KeyI: 33,
  KeyO: 38,
  KeyP: 41,
  BracketLeft: 46,
  BracketRight: 49,
  Enter: 1,
  CapsLock: 52,
  KeyA: 10,
  KeyS: 13,
  KeyD: 18,
  KeyF: 21,
  KeyG: 26,
  KeyH: 29,
  KeyJ: 34,
  KeyK: 37,
  KeyL: 42,
  Semicolon: 45,
  Quote: 50,
  ShiftLeft: 15,
  KeyZ: 12,
  KeyX: 23,
  KeyC: 20,
  KeyV: 31,
  KeyB: 28,
  KeyN: 39,
  KeyM: 36,
  Comma: 47,
  Period: 44,
  Slash: 55,
  ControlLeft: 61,
  Space: 60,
};

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
    this.keydown = this.keydown.bind(this);
    this.keyup = this.keyup.bind(this);

    window.addEventListener('keydown', e => {
      this.keydown(e.code);
      e.preventDefault();
    });
    window.addEventListener('keyup', e => {
      this.keyup(e.code);
    });
  }

  keydown(key) {
    if (key in KEYBOARD) {
      const code = KEYBOARD[key];
      this.mod.keydown(this.c64, code);
    }
  }

  keyup(key) {
    if (key in KEYBOARD) {
      const code = KEYBOARD[key];
      this.mod.keyup(this.c64, code);
    }
  }

  step() {
    this.mod.stepVM(this.c64);
  }

  frame(ms) {
    if (this._lastFrame === 0) {
      this._lastFrame = ms;
    }
    const delta = Math.min(ms - this._lastFrame, 100);
    this._lastFrame = ms;

    // collect input

    // update cpu state
    this.mod.runVMFor(this.c64, delta);

    // draw screen
    this.graphics.loadCharMem(this.mem.char);
    this.graphics.loadScreenMem(this.mem.screen);
    this.graphics.loadColorMem(this.mem.color);
    this.graphics.setColors(
      this.mod.getBorderColor(this.c64),
      this.mod.getBgColor(this.c64)
    );
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
      SP: this.mod.getRegister(this.c64, 4).toString(16),
      PC: this.mod.getRegister(this.c64, 5).toString(16),
    });
  }
}

window.VM = VM;
})();
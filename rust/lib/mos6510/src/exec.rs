use cpu::CPU;
use flags;
use memory::{memory_get_short, Memory};

impl CPU {
  pub fn step(&mut self, mem: &mut Memory) -> u8 {
    let index = self.pc;
    let (byte_len, cycles) = match mem.get_byte(index) {
      0x00 => { // BRK
        self.brk(mem);
        (2, 7)
      },
      
      0x01 => { // ORA (nn,X)
        let addr = self.get_address_indexed_indirect(mem);
        self.ora(mem, addr);
        (2, 6)
      },

      0x02 => { // * KIL
        self.kil();
        (1, 3)
      },

      0x03 => { // * SLO (nn,X)
        (2, 1)
      },

      0x04 => { // * DOP
        (2, 1)
      },

      0x05 => { // ORA nn
        let addr = self.get_address_zeropage(mem);
        self.ora(mem, addr);
        (2, 3)
      },

      0x06 => { // ASL nn
        let addr = self.get_address_zeropage(mem);
        let value = mem.get_byte(addr);
        let result = self.asl(value);
        mem.set_byte(addr, result);
        (2, 5)
      },

      0x07 => { // * SLO nn
        (2, 1)
      },

      0x08 => { // PHP
        self.php(mem);
        (1, 3)
      },

      0x09 => { // ORA #nn
        let addr = self.get_address_immediate();
        self.ora(mem, addr);
        (2, 2)
      },

      0x0a => { // ASL A
        let value = self.acc;
        let result = self.asl(value);
        self.acc = result;
        (1, 2)
      },

      0x0b => { // * ANC #nn
        (2, 1)
      },

      0x0c => { // * TOP
        (3, 1)
      },

      0x0d => { // ORA nnnn
        let addr = self.get_address_absolute(mem);
        self.ora(mem, addr);
        (3, 4)
      },

      0x0e => { // ASL nnnn
        let addr = self.get_address_absolute(mem);
        let value = mem.get_byte(addr);
        let result = self.asl(value);
        mem.set_byte(addr, result);
        (3, 6)
      },

      0x0f => { // * SLO nnnn
        (3, 1)
      },

      0x10 => { // BPL
        if self.status & flags::FLAG_NEGATIVE == 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (2, 3)
        } else {
          (2, 2)
        }
      },

      0x11 => { // ORA (nn),Y
        let addr = self.get_address_indirect_indexed(mem);
        self.ora(mem, addr);
        (2, 5)
      },

      0x12 => { // KIL
        self.kil();
        (1, 3)
      },

      0x13 => { // * SLO (nn),Y
        (2, 1)
      },

      0x14 => { // * DOP
        (2, 2)
      },

      0x15 => { // ORA nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.ora(mem, addr);
        (2, 4)
      },

      0x16 => { // ASL nn,X
        let addr = self.get_address_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let result = self.asl(value);
        mem.set_byte(addr, result);
        (2, 6)
      },

      0x17 => { // * SLO nn,Y
        (2, 1)
      },

      0x18 => { // CLC
        self.clc();
        (1, 2)
      },

      0x19 => { // ORA nnnn,Y
        let addr = self.get_address_absolute_y(mem);
        self.ora(mem, addr);
        (3, 4)
      },

      0x1a => { // * NOP
        (1, 1)
      },

      0x1b => { // * SLO nnnn,Y
        (3, 1)
      },

      0x1c => { // * NOP nnnn,X
        (3, 1)
      },

      0x1d => { // ORA nnnn,X
        let addr = self.get_address_absolute_x(mem);
        self.ora(mem, addr);
        (3, 4)
      },

      0x1e => { // ASL nnnn,X
        let addr = self.get_address_absolute_x(mem);
        let value = mem.get_byte(addr);
        let result = self.asl(value);
        mem.set_byte(addr, result);
        (3, 7)
      },

      0x1f => { // * SLO nnnn,X
        (3, 1)
      },

      0x20 => { // JSR
        let ret = self.pc + 2;
        self.push(mem, (ret >> 8) as u8);
        self.push(mem, (ret & 0xff) as u8);
        let dest = memory_get_short(mem, self.pc + 1);
        self.pc = dest;
        (0, 6)
      },

      0x21 => { // AND (nn,X)
        let addr = self.get_address_indexed_indirect(mem);
        self.and(mem, addr);
        (2, 6)
      },

      0x22 => { // KIL
        self.kil();
        (1, 3)
      },

      0x23 => { // * RLA (nn,X)
        (2, 1)
      },

      0x24 => { // BIT nn
        let addr = self.get_address_zeropage(mem);
        self.bit(mem, addr);
        (2, 3)
      },

      0x25 => { // AND nn
        let addr = self.get_address_zeropage(mem);
        self.and(mem, addr);
        (2, 3)
      },

      0x26 => { // ROL nn
        let addr = self.get_address_zeropage(mem);
        let value = mem.get_byte(addr);
        let result = self.rol(value);
        mem.set_byte(addr, result);
        (2, 5)
      },

      0x27 => { // * RLA nn
        (2, 1)
      },

      0x28 => { // PLP
        self.plp(mem);
        (1, 4)
      },

      0x29 => { // AND #nn
        let addr = self.get_address_immediate();
        self.and(mem, addr);
        (2, 2)
      },

      0x2a => { // ROL A
        let value = self.acc;
        let result = self.rol(value);
        self.acc = result;
        (1, 2)
      },

      0x2b => { // * ANC #nn
        (2, 1)
      },

      0x2c => { // BIT nnnn
        let addr = self.get_address_absolute(mem);
        self.bit(mem, addr);
        (3, 4)
      },

      0x2d => { // AND nnnn
        let addr = self.get_address_absolute(mem);
        self.and(mem, addr);
        (3, 4)
      },

      0x2e => { // ROL nnnn
        let addr = self.get_address_absolute(mem);
        let value = mem.get_byte(addr);
        let result = self.rol(value);
        mem.set_byte(addr, result);
        (3, 6)
      },

      0x2f => { // * RLA nnnnn
        (3, 1)
      },

      0x30 => { // BMI
        if self.status & flags::FLAG_NEGATIVE > 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (2, 3)
        } else {
          (2, 2)
        }
      },

      0x31 => { // AND (nn),Y
        let addr = self.get_address_indirect_indexed(mem);
        self.and(mem, addr);
        (2, 5)
      },

      0x32 => { // KIL
        self.kil();
        (1, 3)
      },

      0x33 => { // * RLA (nn),Y
        (2, 1)
      },

      0x34 => { // * DOP
        (2, 2)
      },

      0x35 => { // AND nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.and(mem, addr);
        (2, 4)
      },

      0x36 => { // ROL nn,X
        let addr = self.get_address_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let result = self.rol(value);
        mem.set_byte(addr, result);
        (2, 6)
      },

      0x37 => { // * RLA nn,Y
        (2, 1)
      },

      0x38 => { // SEC
        self.sec();
        (1, 2)
      },

      0x39 => { // AND nnnn,Y
        let addr = self.get_address_absolute_y(mem);
        self.and(mem, addr);
        (3, 4)
      },

      0x3a => { // * NOP
        (1, 1)
      },

      0x3b => { // * RLA nnnn,Y
        (3, 1)
      },

      0x3c => { // * TOP
        (3, 1)
      },

      0x3d => { // AND nnnn,X
        let addr = self.get_address_absolute_x(mem);
        self.and(mem, addr);
        (3, 4)
      },

      0x3e => { // ROL nnnn,X
        let addr = self.get_address_absolute_x(mem);
        let value = mem.get_byte(addr);
        let result = self.rol(value);
        mem.set_byte(addr, result);
        (3, 7)
      },

      0x3f => { // * RLA nnnn,X
        (3, 1)
      },

      0x40 => { // RTI
        self.rti(mem);
        (0, 6)
      },

      0x41 => { // EOR (nn,X)
        let addr = self.get_address_indexed_indirect(mem);
        self.eor(mem, addr);
        (2, 6)
      },

      0x42 => { // KIL
        self.kil();
        (1, 3)
      },

      0x43 => { // * SRE (nn,X)
        (2, 1)
      },

      0x44 => { // * DOP
        (2, 1)
      },

      0x45 => { // EOR nn
        let addr = self.get_address_zeropage(mem);
        self.eor(mem, addr);
        (2, 3)
      },

      0x46 => { // LSR nn
        let addr = self.get_address_zeropage(mem);
        let value = mem.get_byte(addr);
        let result = self.lsr(value);
        mem.set_byte(addr, result);
        (2, 5)
      },

      0x47 => { // * SRE nn
        (2, 1)
      },

      0x48 => { // PHA
        self.pha(mem);
        (1, 3)
      },

      0x49 => { // EOR #nn
        let addr = self.get_address_immediate();
        self.eor(mem, addr);
        (2, 2)
      },

      0x4a => { // LSR A
        let value = self.acc;
        let result = self.lsr(value);
        self.acc = result;
        (1, 2)
      },

      0x4b => { // * ALR #nn
        (2, 1)
      },

      0x4c => { // JMP nnnn
        let dest = memory_get_short(mem, self.pc + 1);
        self.pc = dest;
        (0, 3)
      },

      0x4d => { // EOR nnnn
        let addr = self.get_address_absolute(mem);
        self.eor(mem, addr);
        (2, 4)
      },

      0x4e => { // LSR nnnn
        let addr = self.get_address_absolute(mem);
        let value = mem.get_byte(addr);
        let result = self.lsr(value);
        mem.set_byte(addr, result);
        (3, 6)
      },

      0x4f => { // * SRE nnnn
        (3, 1)
      },

      0x50 => { // BVC
        if self.status & flags::FLAG_OVERFLOW == 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (2, 3)
        } else {
          (2, 2)
        }
      },

      0x51 => { // EOR (nn),Y
        let addr = self.get_address_indirect_indexed(mem);
        self.eor(mem, addr);
        (2, 5)
      },

      0x52 => { // KIL
        self.kil();
        (1, 3)
      },

      0x53 => { // * SRE (nn),Y
        (2, 1)
      },

      0x54 => { // * DOP
        (2, 1)
      },

      0x55 => { // EOR nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.eor(mem, addr);
        (2, 4)
      },

      0x56 => { // LSR nn,X
        let addr = self.get_address_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let result = self.lsr(value);
        mem.set_byte(addr, result);
        (2, 6)
      },

      0x57 => { // * SRE nn,X
        (2, 1)
      },

      0x58 => { // CLI
        self.cli();
        (1, 2)
      },

      0x59 => { // EOR nnnn,Y
        let addr = self.get_address_absolute_y(mem);
        self.eor(mem, addr);
        (3, 4)
      },

      0x5a => { // * NOP
        (1, 1)
      },

      0x5b => { // * SRE nnnn,Y
        (3, 1)
      },

      0x5c => { // * TOP
        (3, 1)
      },

      0x5d => { // EOR nnnn,X
        let addr = self.get_address_absolute_x(mem);
        self.eor(mem, addr);
        (3, 4)
      },

      0x5e => { // LSR nnnn,X
        let addr = self.get_address_absolute_x(mem);
        let value = mem.get_byte(addr);
        let result = self.lsr(value);
        mem.set_byte(addr, result);
        (3, 7)
      },

      0x5f => { // * SRE nnnnX
        (3, 1)
      },

      0x60 => { // RTS
        self.rts(mem);
        (1, 6)
      },

      0x61 => { // ADC (nn,X)
        let addr = self.get_address_indexed_indirect(mem);
        let value = mem.get_byte(addr);
        self.adc(value);
        (2, 6)
      },

      0x62 => { // KIL
        self.kil();
        (1, 3)
      },

      0x63 => { // * RRA (nn,X)
        (2, 1)
      },

      0x64 => { // * DOP
        (2, 1)
      },

      0x65 => { // ADC nn
        let addr = self.get_address_zeropage(mem);
        let value = mem.get_byte(addr);
        self.adc(value);
        (2, 3)
      },

      0x66 => { // ROR nn
        let addr = self.get_address_zeropage(mem);
        let value = mem.get_byte(addr);
        let result = self.ror(value);
        mem.set_byte(addr, result);
        (2, 5)
      },

      0x67 => { // * RRA nn
        (2, 1)
      },

      0x68 => { // PLA
        self.pla(mem);
        (1, 4)
      },

      0x69 => { // ADC #nn
        let addr = self.get_address_immediate();
        let value = mem.get_byte(addr);
        self.adc(value);
        (2, 2)
      },

      0x6a => { // ROR A
        let value = self.acc;
        let result = self.ror(value);
        self.acc = result;
        (1, 2)
      },

      0x6b => { // * ARR #nn
        (2, 1)
      },

      0x6c => { // JMP (nnnn)
        let source = memory_get_short(mem, self.pc + 1);
        let dest = memory_get_short(mem, source);
        self.pc = dest;
        (0, 3)
      },

      0x6d => { // ADC nnnn
        let addr = self.get_address_absolute(mem);
        let value = mem.get_byte(addr);
        self.adc(value);
        (3, 4)
      },

      0x6e => { // ROR nnnn
        let addr = self.get_address_absolute(mem);
        let value = mem.get_byte(addr);
        let result = self.ror(value);
        mem.set_byte(addr, result);
        (3, 6)
      },

      0x6f => { // * RRA nnnn
        (3, 1)
      },

      0x70 => { // BVS
        if self.status & flags::FLAG_OVERFLOW > 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (2, 3)
        } else {
          (2, 2)
        }
      },

      0x71 => { // ADC (nn),Y
        let addr = self.get_address_indirect_indexed(mem);
        let value = mem.get_byte(addr);
        self.adc(value);
        (2, 6)
      },

      0x72 => { // KIL
        self.kil();
        (1, 3)
      },

      0x73 => { // * RRA (nn),Y
        (2, 1)
      },

      0x74 => { // * DOP
        (2, 1)
      },

      0x75 => { // ADC nn,X
        let addr = self.get_address_zeropage_x(mem);
        let value = mem.get_byte(addr);
        self.adc(value);
        (2, 3)
      },

      0x76 => { // ROR nn,X
        let addr = self.get_address_zeropage_x(mem);
        let value = mem.get_byte(addr);
        let result = self.ror(value);
        mem.set_byte(addr, result);
        (2, 6)
      },

      0x77 => { // * RRA nn,X
        (2, 1)
      },

      0x78 => { // SEI
        self.sei();
        (1, 2)
      },

      0x79 => { // ADC nnnn,Y
        let addr = self.get_address_absolute_y(mem);
        let value = mem.get_byte(addr);
        self.adc(value);
        (3, 4)
      },

      0x7a => { // * NOP
        (1, 1)
      },

      0x7b => { // * RRA nnnn,Y
        (3, 1)
      },

      0x7c => { // * TOP
        (3, 1)
      },

      0x7d => { // ADC nnnn,X
        let addr = self.get_address_absolute_x(mem);
        let value = mem.get_byte(addr);
        self.adc(value);
        (3, 4)
      },

      0x7e => { // ROR nnnn,X
        let addr = self.get_address_absolute_x(mem);
        let value = mem.get_byte(addr);
        let result = self.ror(value);
        mem.set_byte(addr, result);
        (3, 7)
      },

      0x7f => { // * RRA nnnn,X
        (3, 1)
      },

      0x80 => { // * NOP
        (1, 1)
      },

      0x81 => { // STA (nn,X)
        let addr = self.get_address_indexed_indirect(mem);
        self.sta(mem, addr);
        (2, 6)
      },

      0x82 => { // * NOP
        (1, 1)
      },

      0x83 => { // * SAX (nn,X)
        (2, 6)
      },

      0x84 => { // STY nn
        let addr = self.get_address_zeropage(mem);
        self.sty(mem, addr);
        (2, 3)
      },

      0x85 => { // STA nn
        let addr = self.get_address_zeropage(mem);
        self.sta(mem, addr);
        (2, 3)
      },

      0x86 => { // STX nn
        let addr = self.get_address_zeropage(mem);
        self.stx(mem, addr);
        (2, 3)
      },

      0x87 => { // * SAX nn
        (2, 1)
      },

      0x88 => { // DEY
        self.dey();
        (1, 2)
      },

      0x89 => { // * NOP
        (1, 1)
      },

      0x8a => { // TXA
        self.txa();
        (1, 2)
      },

      0x8b => { // * XXA #nn
        (2, 1)
      },

      0x8c => { // STY nnnn
        let addr = self.get_address_absolute(mem);
        self.sty(mem, addr);
        (3, 4)
      },

      0x8d => { // STA nnnn
        let addr = self.get_address_absolute(mem);
        self.sta(mem, addr);
        (3, 4)
      },

      0x8e => { // STX nnnn
        let addr = self.get_address_absolute(mem);
        self.stx(mem, addr);
        (3, 4)
      },

      0x8f => { // * SAX nnnn
        (3, 1)
      },

      0x90 => { // BCC
        if self.status & flags::FLAG_CARRY == 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (2, 3)
        } else {
          (2, 2)
        }
      },

      0x91 => { // STA (nn),Y
        let addr = self.get_address_indirect_indexed(mem);
        self.sta(mem, addr);
        (2, 6)
      },

      0x92 => { // KIL
        self.kil();
        (1, 3)
      },

      0x93 => { // * AHX (nn),Y
        (2, 1)
      },

      0x94 => { // STY nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.sty(mem, addr);
        (2, 4)
      },

      0x95 => { // STA nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.sta(mem, addr);
        (2, 4)
      },

      0x96 => { // STX nn,Y
        let addr = self.get_address_zeropage_y(mem);
        self.stx(mem, addr);
        (2, 4)
      },

      0x97 => { // * SAX nn,Y
        (2, 1)
      },

      0x98 => { // TYA
        self.tya();
        (1, 2)
      },

      0x99 => { // STA nnnn,Y
        let addr = self.get_address_absolute_y(mem);
        self.sta(mem, addr);
        (3, 5)
      },

      0x9a => { // TXS
        self.txs();
        (1, 2)
      },

      0x9b => { // * TAS nnnn,Y
        (3, 1)
      },

      0x9c => { // * SHY nnnn,X
        (3, 1)
      },

      0x9d => { // STA nnnn,X
        let addr = self.get_address_absolute_x(mem);
        self.sta(mem, addr);
        (3, 5)
      },

      0x9e => { // * SHX
        (1, 1)
      },

      0x9f => { // * AHX nnnn,Y
        (3, 1)
      },

      0xa0 => { // LDY #nn
        let addr = self.get_address_immediate();
        self.ldy(mem, addr);
        (2, 2)
      },

      0xa1 => { // LDA (nn,X)
        let addr = self.get_address_indexed_indirect(mem);
        self.lda(mem, addr);
        (2, 6)
      },

      0xa2 => { // LDX #nn
        let addr = self.get_address_immediate();
        self.ldx(mem, addr);
        (2, 2)
      },

      0xa3 => { // * LAX (nn,X)
        (2, 1)
      },

      0xa4 => { // LDY nn
        let addr = self.get_address_zeropage(mem);
        self.ldy(mem, addr);
        (2, 2)
      },

      0xa5 => { // LDA nn
        let addr = self.get_address_zeropage(mem);
        self.lda(mem, addr);
        (2, 2)
      },

      
      0xa6 => { // LDX nn
        let addr = self.get_address_zeropage(mem);
        self.ldx(mem, addr);
        (2, 2)
      },

      0xa7 => { // * LAX nn
        (2, 1)
      },

      0xa8 => { // TAY
        self.tay();
        (1, 2)
      },

      0xa9 => { // LDA #nn
        let addr = self.get_address_immediate();
        self.lda(mem, addr);
        (2, 2)
      },

      0xaa => { // TAX
        self.tax();
        (1, 2)
      },

      0xab => { // LAX #nn
        (2, 1)
      },

      0xac => { // LDY nnnn
        let addr = self.get_address_absolute(mem);
        self.ldy(mem, addr);
        (3, 4)
      },

      0xad => { // LDA nnnn
        let addr = self.get_address_absolute(mem);
        self.lda(mem, addr);
        (3, 4)
      },

      0xae => { // LDX nnnn
        let addr = self.get_address_absolute(mem);
        self.ldx(mem, addr);
        (3, 4)
      },

      0xaf => { // * LAX nnnn
        (3, 1)
      },

      0xb0 => { // BCS
        if self.status & flags::FLAG_CARRY > 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (2, 3)
        } else {
          (2, 2)
        }
      },

      0xb1 => { // LDA (nn),Y
        let addr = self.get_address_indirect_indexed(mem);
        self.lda(mem, addr);
        (2, 5)
      },

      0xb2 => { // KIL
        self.kil();
        (1, 3)
      },

      0xb3 => { // * LAX (nn),Y
        (2, 1)
      },

      0xb4 => { // LDY nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.ldy(mem, addr);
        (2, 4)
      },

      0xb5 => { // LDA nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.lda(mem, addr);
        (2, 4)
      },

      0xb6 => { // LDX nn,Y
        let addr = self.get_address_zeropage_y(mem);
        self.ldx(mem, addr);
        (2, 4)
      },

      0xb7 => { // * LAX nn,Y
        (2, 1)
      },

      0xb8 => { // CLV
        self.clv();
        (1, 2)
      },

      0xb9 => { // LDA nnnn,Y
        let addr = self.get_address_absolute_y(mem);
        self.lda(mem, addr);
        (3, 4)
      },

      0xba => { // TSX
        self.tsx();
        (1, 2)
      },

      0xbb => { // * LAS nnnn,Y
        (3, 1)
      },

      0xbc => { // LDY nnnn,X
        let addr = self.get_address_absolute_x(mem);
        self.ldy(mem, addr);
        (3, 4)
      },

      0xbd => { // LDA nnnn,X
        let addr = self.get_address_absolute_x(mem);
        self.lda(mem, addr);
        (3, 4)
      },

      0xbe => { // LDX nnnn,Y
        let addr = self.get_address_absolute_y(mem);
        self.ldx(mem, addr);
        (3, 4)
      },

      0xbf => { // * LAX nnnn,Y
        (3, 1)
      },

      0xc0 => { // CPY #nn
        let addr = self.get_address_immediate();
        self.cpy(mem, addr);
        (2, 2)
      },

      0xc1 => { // CMP (nn,X)
        let addr = self.get_address_indexed_indirect(mem);
        self.cmp(mem, addr);
        (2, 6)
      },

      0xc2 => { // * DOP
        (2, 1)
      },

      0xc3 => { // * DCP (nn,X)
        (2, 1)
      },

      0xc4 => { // CPY nn
        let addr = self.get_address_zeropage(mem);
        self.cpy(mem, addr);
        (2, 3)
      },

      0xc5 => { // CMP nn
        let addr = self.get_address_zeropage(mem);
        self.cmp(mem, addr);
        (2, 3)
      },

      0xc6 => { // DEC nn
        let addr = self.get_address_zeropage(mem);
        self.dec(mem, addr);
        (2, 5)
      },

      0xc7 => { // * DCP nn
        (2, 1)
      },

      0xc8 => { // INY
        self.iny();
        (1, 2)
      },

      0xc9 => { // CMP #nn
        let addr = self.get_address_immediate();
        self.cmp(mem, addr);
        (2, 2)
      },

      0xca => { // DEX
        self.dex();
        (1, 2)
      },

      0xcb => { // * AXS #nn
        (2, 1)
      },

      0xcc => { // CPY nnnn
        let addr = self.get_address_absolute(mem);
        self.cpy(mem, addr);
        (3, 4)
      },

      0xcd => { // CMP nnnn
        let addr = self.get_address_absolute(mem);
        self.cmp(mem, addr);
        (3, 4)
      },

      0xce => { // DEC nnnn
        let addr = self.get_address_absolute(mem);
        self.dec(mem, addr);
        (3, 6)
      },

      0xcf => { // * DCP nnnn
        (3, 1)
      },

      0xd0 => { // BNE
        if self.status & flags::FLAG_ZERO == 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (2, 3)
        } else {
          (2, 2)
        }
      },

      0xd1 => { // CMP (nn),Y
        let addr = self.get_address_indirect_indexed(mem);
        self.cmp(mem, addr);
        (2, 5)
      },

      0xd2 => { // KIL
        self.kil();
        (1, 3)
      },

      0xd3 => { // * ISC (nn,X)
        (2, 1)
      },

      0xd4 => { // * DOP
        (2, 1)
      },

      0xd5 => { // CMP nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.cmp(mem, addr);
        (2, 4)
      },

      0xd6 => { // DEC nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.dec(mem, addr);
        (2, 6)
      },

      0xd7 => { // * DCP nn,X
        (2, 1)
      },

      0xd8 => { // CLD
        self.cld();
        (1, 2)
      },

      0xd9 => { // CMP nnnn,Y
        let addr = self.get_address_absolute_y(mem);
        self.cmp(mem, addr);
        (3, 4)
      },

      0xda => { // * NOP
        (1, 1)
      },

      0xdb => { // * DCP nnnn,Y
        (3, 1)
      },

      0xdc => { // * TOP
        (3, 1)
      },

      0xdd => { // CMP nnnn,X
        let addr = self.get_address_absolute_x(mem);
        self.cmp(mem, addr);
        (3, 4)
      },

      0xde => { // DEC nnnn,X
        let addr = self.get_address_absolute_x(mem);
        self.dec(mem, addr);
        (3, 7)
      },

      0xdf => { // * DCP nnnn,X
        (3, 1)
      },

      0xe0 => { // CPX #nn
        let addr = self.get_address_immediate();
        self.cpx(mem, addr);
        (2, 2)
      },

      0xe1 => { // SBC (nn,X)
        let addr = self.get_address_indexed_indirect(mem);
        self.sbc(mem, addr);
        (2, 6)
      },

      0xe2 => { // * DOP
        (2, 1)
      },

      0xe3 => { // * ISC (nn,X)
        (2, 1)
      },

      0xe4 => { // CPX nn
        let addr = self.get_address_zeropage(mem);
        self.cpx(mem, addr);
        (2, 3)
      },

      0xe5 => { // SBC nn
        let addr = self.get_address_zeropage(mem);
        self.sbc(mem, addr);
        (2, 3)
      },

      0xe6 => { // INC nn
        let addr = self.get_address_zeropage(mem);
        self.inc(mem, addr);
        (2, 5)
      },

      0xe7 => { // * ISC nn
        (2, 1)
      },

      0xe8 => { // INX
        self.inx();
        (1, 2)
      },

      0xe9 => { // SBC #nn
        let addr = self.get_address_immediate();
        self.sbc(mem, addr);
        (2, 2)
      },

      0xea => { // NOP
        (1, 2)
      },

      0xeb => { // * SBC #nn
        (2, 1)
      },

      0xec => { // CPX nnnn
        let addr = self.get_address_absolute(mem);
        self.cpx(mem, addr);
        (3, 4)
      },

      0xed => { // SBC nnnn
        let addr = self.get_address_absolute(mem);
        self.sbc(mem, addr);
        (3, 4)
      },

      0xee => { // INC nnnn
        let addr = self.get_address_absolute(mem);
        self.inc(mem, addr);
        (3, 6)
      },

      0xef => { // * ISC nnnn
        (3, 1)
      },

      0xf0 => { // BEQ
        if self.status & flags::FLAG_ZERO > 0 {
          let addr_offset = mem.get_byte(self.pc + 1);
          self.jump_pc(addr_offset);
          (2, 3)
        } else {
          (2, 2)
        }
      },

      0xf1 => { // SBC (nn),Y
        let addr = self.get_address_indirect_indexed(mem);
        self.sbc(mem, addr);
        (2, 5)
      },

      0xf2 => { // * KIL
        self.kil();
        (1, 3)
      },

      0xf3 => { // * ISC (nn),Y
        (2, 1)
      },

      0xf4 => { // * DOP
        (2, 1)
      },

      0xf5 => { // SBC nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.sbc(mem, addr);
        (2, 4)
      },

      0xf6 => { // INC nn,X
        let addr = self.get_address_zeropage_x(mem);
        self.inc(mem, addr);
        (2, 6)
      },

      0xf7 => { // * ISC nn,X
        (2, 1)
      },

      0xf8 => { // SED
        self.sed();
        (1, 2)
      },

      0xf9 => { // SBC nnnn,Y
        let addr = self.get_address_absolute_y(mem);
        self.sbc(mem, addr);
        (3, 4)
      },

      0xfa => { // * NOP
        (1, 1)
      },

      0xfb => { // * ISC nnnn,Y
        (3, 1)
      },

      0xfc => { // TOP
        (3, 1)
      },

      0xfd => { // SBC nnnn,X
        let addr = self.get_address_absolute_x(mem);
        self.sbc(mem, addr);
        (3, 4)
      },

      0xfe => { // INC nnnn,X
        let addr = self.get_address_absolute_x(mem);
        self.inc(mem, addr);
        (3, 7)
      },

      0xff => { // * ISC nnnn,X
        (3, 1)
      },
    };
    self.pc += byte_len;
    cycles
  }
}

#[cfg(test)]
mod tests {
  use cpu::CPU;
  use memory::Memory;
  use memory::mock::MockMem;

  #[test]
  fn subroutine_and_return() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x1000;
    mem.set_byte(0x1000, 0x20); // JSR
    mem.set_byte(0x1001, 0x50);
    mem.set_byte(0x1002, 0x12);
    mem.set_byte(0x1250, 0xea); // NOP
    mem.set_byte(0x1251, 0x60); // RTS
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x1250);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x1251);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x1003);
  }

  #[test]
  fn instruction_0x01() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.acc = 5;
    cpu.x = 8;
    mem.set_byte(0x100, 0x01);
    mem.set_byte(0x101, 0xe0);
    mem.set_byte(0xe8, 0x45);
    mem.set_byte(0xe9, 0x11);
    mem.set_byte(0x1145, 6);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x102);
    assert_eq!(cpu.acc, 7);
  }

  #[test]
  fn instruction_0x05() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.acc = 0xf0;
    mem.set_byte(0x100, 0x05);
    mem.set_byte(0x101, 0xa4);
    mem.set_byte(0xa4, 0x15);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x102);
    assert_eq!(cpu.acc, 0xf5);
  }


  #[test]
  fn instruction_0x0d() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.acc = 0x11;
    mem.set_byte(0x100, 0x0d);
    mem.set_byte(0x101, 0xb0);
    mem.set_byte(0x102, 0x0b);
    mem.set_byte(0x0bb0, 0x22);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x103);
    assert_eq!(cpu.acc, 0x33);
  }

  #[test]
  fn instruction_0x10() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.status = 0;
    mem.set_byte(0x100, 0x10);
    mem.set_byte(0x101, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x107);
    cpu.status = 1 << 7;
    mem.set_byte(0x107, 0x10);
    mem.set_byte(0x108, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.pc, 0x109);
  }

  #[test]
  fn instruction_0x24() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    mem.set_byte(0x005, 0x10);
    mem.set_byte(0x100, 0x24);
    mem.set_byte(0x101, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 2);
    cpu.acc = 0x30;
    mem.set_byte(0x102, 0x24);
    mem.set_byte(0x103, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 0);
    mem.set_byte(0x005, 0x70);
    mem.set_byte(0x104, 0x24);
    mem.set_byte(0x105, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 1 << 6);
    mem.set_byte(0x005, 0xb0);
    mem.set_byte(0x106, 0x24);
    mem.set_byte(0x107, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, 1 << 7);
    mem.set_byte(0x005, 0xd0);
    mem.set_byte(0x108, 0x24);
    mem.set_byte(0x109, 0x05);
    cpu.step(&mut mem);
    assert_eq!(cpu.status, (1 << 7) + (1 << 6));
  }

  #[test]
  fn instruction_0x2a() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.status = 1;
    cpu.acc = 0b01101100;
    mem.set_byte(0x100, 0x2a);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0b11011001);
    assert_eq!(cpu.status & 1, 0);
    mem.set_byte(0x101, 0x2a);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0b10110010);
    assert_eq!(cpu.status & 1, 1);
  }

  #[test]
  fn instruction_0x65() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.acc = 0x22;
    mem.set_byte(0x40, 0x33);
    mem.set_byte(0x100, 0x65);
    mem.set_byte(0x101, 0x40);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x55);
  }

  #[test]
  fn instruction_0x66() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    mem.set_byte(0x40, 0b10110011);
    mem.set_byte(0x100, 0x66);
    mem.set_byte(0x101, 0x40);
    cpu.step(&mut mem);
    assert_eq!(mem.get_byte(0x40), 0b01011001);
    assert_eq!(cpu.status, 1);
    mem.set_byte(0x102, 0x66);
    mem.set_byte(0x103, 0x40);
    cpu.step(&mut mem);
    assert_eq!(mem.get_byte(0x40), 0b10101100);
    assert_eq!(cpu.status, 1 | (1 << 7));
  }

  #[test]
  fn instruction_0x69() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    mem.set_byte(0x100, 0x69);
    mem.set_byte(0x101, 0x40);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x40);
    assert_eq!(cpu.status, 0);
    mem.set_byte(0x102, 0x69);
    mem.set_byte(0x103, 0x80);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0xc0);
    assert_eq!(cpu.status, 1 << 7);
    mem.set_byte(0x104, 0x69);
    mem.set_byte(0x105, 0x80);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x40);
    assert_eq!(cpu.status, (1 << 6) + 1);
    mem.set_byte(0x106, 0x69);
    mem.set_byte(0x107, 0x70);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0xb1);
    assert_eq!(cpu.status, (1 << 7) + (1 << 6));
  }

  #[test]
  fn instruction_0x6d() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    mem.set_byte(0x124, 0x44);
    mem.set_byte(0x100, 0x6d);
    mem.set_byte(0x101, 0x24);
    mem.set_byte(0x102, 0x01);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x44);
  }

  #[test]
  fn instruction_0x75() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.x = 0x2b;
    mem.set_byte(0x6b, 0x33);
    mem.set_byte(0x100, 0x75);
    mem.set_byte(0x101, 0x40);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x33);
  }

  #[test]
  fn instruction_0x79() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.y = 0x14;
    mem.set_byte(0x138, 0x44);
    mem.set_byte(0x100, 0x79);
    mem.set_byte(0x101, 0x24);
    mem.set_byte(0x102, 0x01);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x44);
  }

  #[test]
  fn instruction_0x7d() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.x = 0x23;
    mem.set_byte(0x147, 0x67);
    mem.set_byte(0x100, 0x7d);
    mem.set_byte(0x101, 0x24);
    mem.set_byte(0x102, 0x01);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x67);
  }

  #[test]
  fn instruction_0x86() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.x = 0x23;
    mem.set_byte(0x100, 0x86);
    mem.set_byte(0x101, 0x44);
    cpu.step(&mut mem);
    assert_eq!(mem.get_byte(0x44), 0x23);
  }

  #[test]
  fn instruction_0xe9() {
    let mut cpu = CPU::new();
    let mut mem = MockMem::new();
    cpu.pc = 0x100;
    cpu.acc = 0x43;
    cpu.status = 1;
    mem.set_byte(0x100, 0xe9);
    mem.set_byte(0x101, 0x12);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x31);
    cpu.status = 0;
    mem.set_byte(0x102, 0xe9);
    mem.set_byte(0x103, 0x4);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0x2c);
    cpu.acc = 0x50;
    cpu.status = 1;
    mem.set_byte(0x104, 0xe9);
    mem.set_byte(0x105, 0xb0);
    cpu.step(&mut mem);
    assert_eq!(cpu.acc, 0xa0);
    assert!(cpu.status & (1 << 6) > 0);
  }
}
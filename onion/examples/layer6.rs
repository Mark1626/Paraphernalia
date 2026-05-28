use anyhow::{Result, anyhow};
use std::fs;

struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    la: u32,
    lb: u32,
    lc: u32,
    ld: u32,
    ptr: u32,
    pc: u32,
}

impl Registers {
    fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            la: 0,
            lb: 0,
            lc: 0,
            ld: 0,
            ptr: 0,
            pc: 0,
        }
    }
}

struct VM {
    memory: Vec<u8>,
    registers: Registers,
}

impl VM {
    fn new(memory: Vec<u8>) -> Self {
        Self {
            memory: memory,
            registers: Registers::new(),
        }
    }

    fn decode(&mut self) -> Instruction {
        match self.memory[self.registers.pc as usize] {
            /* ADD */ 0xC2 => Instruction::ADD,
            /* SUB */ 0xC3 => Instruction::SUB,
            /* XOR */ 0xC4 => Instruction::XOR,
            /* OUT */ 0x02 => Instruction::OUT,
            /* HALT */ 0x01 => Instruction::HALT,
            /* CMP */ 0xC1 => Instruction::CMP,
            /* APTR */
            0xE1 => Instruction::APTR(self.memory[(self.registers.pc + 1) as usize]),
            /* JEZ */
            0x21 => {
                let addr = u32::from_le_bytes([
                    self.memory[(self.registers.pc + 1) as usize],
                    self.memory[(self.registers.pc + 2) as usize],
                    self.memory[(self.registers.pc + 3) as usize],
                    self.memory[(self.registers.pc + 4) as usize],
                ]);
                Instruction::JEZ(addr)
            }
            /* JNZ */
            0x22 => {
                let addr = u32::from_le_bytes([
                    self.memory[(self.registers.pc + 1) as usize],
                    self.memory[(self.registers.pc + 2) as usize],
                    self.memory[(self.registers.pc + 3) as usize],
                    self.memory[(self.registers.pc + 4) as usize],
                ]);
                Instruction::JNZ(addr)
            }
            opcode => {
                let op = opcode >> 6;
                return match op {
                    /* MV and MVI */
                    0b01 => {
                        let dst = (opcode >> 3) & 0b111;
                        let src = opcode & 0b111;
                        if src == 0 {
                            let inc = self.memory[(self.registers.pc + 1) as usize];
                            Instruction::MVI(dst, inc)
                        } else {
                            Instruction::MV(dst, src)
                        }
                    }
                    /* MV32 anv MVI32 */
                    0b10 => {
                        let dst = (opcode >> 3) & 0b111;
                        let src = opcode & 0b111;
                        if src == 0 {
                            let inc = u32::from_le_bytes([
                                self.memory[(self.registers.pc + 1) as usize],
                                self.memory[(self.registers.pc + 2) as usize],
                                self.memory[(self.registers.pc + 3) as usize],
                                self.memory[(self.registers.pc + 4) as usize],
                            ]);
                            Instruction::MVI32(dst, inc)
                        } else {
                            Instruction::MV32(dst, src)
                        }
                    }
                    _ => Instruction::NOP,
                };
            }
        }
    }

    fn execute(&mut self, out: &mut Vec<u8>) -> bool {
        // Halt after 1_000_000 instruction as a safe guard
        for _ in 1..1_000_000 {
            let inst = self.decode();

            self.registers.pc += inst.byte_size() as u32;

            match inst {
                Instruction::ADD => {
                    self.registers.a = self.registers.a.wrapping_add(self.registers.b);
                }
                Instruction::SUB => {
                    self.registers.a = self.registers.a.wrapping_sub(self.registers.b);
                }
                Instruction::XOR => {
                    self.registers.a = self.registers.a ^ self.registers.b;
                }
                Instruction::APTR(imm8) => {
                    self.registers.ptr = self.registers.ptr.wrapping_add(imm8 as u32);
                }
                Instruction::CMP => {
                    if self.registers.a == self.registers.b {
                        self.registers.f = 0;
                    } else {
                        self.registers.f = 1;
                    }
                }
                Instruction::HALT => {
                    return true;
                }
                Instruction::JEZ(imm32) => {
                    if self.registers.f == 0 {
                        self.registers.pc = imm32;
                    }
                }
                Instruction::JNZ(imm32) => {
                    if self.registers.f != 0 {
                        self.registers.pc = imm32;
                    }
                }
                Instruction::MVI(dest, idx) => match dest {
                    1 => {
                        self.registers.a = idx;
                    }
                    2 => {
                        self.registers.b = idx;
                    }
                    3 => {
                        self.registers.c = idx;
                    }
                    4 => {
                        self.registers.d = idx;
                    }
                    5 => {
                        self.registers.e = idx;
                    }
                    6 => {
                        self.registers.f = idx;
                    }
                    7 => {
                        self.memory[self.registers.ptr as usize + self.registers.c as usize] = idx;
                    }
                    _ => {
                        eprintln!("MVI unknown hit");
                    }
                },
                Instruction::MV(dest, src) => {
                    let src_data = match src {
                        1 => self.registers.a,
                        2 => self.registers.b,
                        3 => self.registers.c,
                        4 => self.registers.d,
                        5 => self.registers.e,
                        6 => self.registers.f,
                        7 => self.memory[self.registers.ptr as usize + self.registers.c as usize],
                        _ => unreachable!("MV unknown arm"),
                    };

                    match dest {
                        1 => {
                            self.registers.a = src_data;
                        }
                        2 => {
                            self.registers.b = src_data;
                        }
                        3 => {
                            self.registers.c = src_data;
                        }
                        4 => {
                            self.registers.d = src_data;
                        }
                        5 => {
                            self.registers.e = src_data;
                        }
                        6 => {
                            self.registers.f = src_data;
                        }
                        7 => {
                            self.memory[self.registers.ptr as usize + self.registers.c as usize] =
                                src_data;
                        }
                        _ => {
                            eprintln!("NOP hit")
                        }
                    }
                }
                Instruction::MVI32(dest, idx) => match dest {
                    1 => {
                        self.registers.la = idx;
                    }
                    2 => {
                        self.registers.lb = idx;
                    }
                    3 => {
                        self.registers.lc = idx;
                    }
                    4 => {
                        self.registers.ld = idx;
                    }
                    5 => {
                        self.registers.ptr = idx;
                    }
                    6 => {
                        self.registers.pc = idx;
                    }
                    _ => {
                        eprintln!("MV32 hit unknown dest")
                    }
                },
                Instruction::MV32(dest, src) => {
                    let src_data: u32 = match src {
                        1 => self.registers.la,
                        2 => self.registers.lb,
                        3 => self.registers.lc,
                        4 => self.registers.ld,
                        5 => self.registers.ptr,
                        6 => self.registers.pc,
                        _ => unreachable!("MV unknown arm"),
                    };

                    match dest {
                        1 => {
                            self.registers.la = src_data;
                        }
                        2 => {
                            self.registers.lb = src_data;
                        }
                        3 => {
                            self.registers.lc = src_data;
                        }
                        4 => {
                            self.registers.ld = src_data;
                        }
                        5 => {
                            self.registers.ptr = src_data;
                        }
                        6 => {
                            self.registers.pc = src_data;
                        }
                        _ => {
                            eprintln!("MV32 hit unknown dest")
                        }
                    }
                }
                Instruction::OUT => {
                    out.push(self.registers.a);
                }
                Instruction::NOP => {
                    eprintln!("NOP hit")
                }
            }
        }
        return false;
    }
}

enum Instruction {
    ADD,
    SUB,
    XOR,
    APTR(u8),
    CMP,
    HALT,
    JEZ(u32),
    JNZ(u32),
    MV(u8, u8),
    MV32(u8, u8),
    MVI(u8, u8),
    MVI32(u8, u32),
    OUT,
    NOP,
}

impl Instruction {
    fn byte_size(&self) -> u32 {
        match self {
            Instruction::ADD => 1,
            Instruction::SUB => 1,
            Instruction::XOR => 1,
            Instruction::APTR(_) => 2,
            Instruction::CMP => 1,
            Instruction::HALT => 1,
            Instruction::JEZ(_) => 5,
            Instruction::JNZ(_) => 5,
            Instruction::MV(_, _) => 1,
            Instruction::MV32(_, _) => 1,
            Instruction::MVI(_, _) => 2,
            Instruction::MVI32(_, _) => 5,
            Instruction::OUT => 1,
            Instruction::NOP => 1,
        }
    }
}

fn decode_layer6(payload: Vec<u8>) -> Result<Vec<u8>> {
    let mut vm = VM::new(payload);
    let mut out_stream: Vec<u8> = Vec::new();
    let result = vm.execute(&mut out_stream);
    if result == false {
        Err(anyhow!("Execution hit max cycles limit"))
    } else {
        Ok(out_stream)
    }
}

fn main() -> Result<()> {
    let input = fs::read_to_string("payload/layer6.data")?;
    let ascii85_decoded = ascii85::decode(&input).map_err(|e| anyhow!("{e:?}"))?;

    let decoded = decode_layer6(ascii85_decoded)?;

    fs::write("layer6_out.data", &decoded)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example program from the Layer 6 spec. Loaded as memory at address 0,
    // it exercises every instruction and outputs the ASCII "Hello, world!".
    #[test]
    fn hello_world_example_program() {
        let program: Vec<u8> = vec![
            0x50, 0x48, // MVI b <- 72
            0xC2, // ADD a <- b
            0x02, // OUT a
            0xA8, 0x4D, 0x00, 0x00, 0x00, // MVI32 ptr <- 0x0000004d
            0x4F, // MV a <- (ptr+c)
            0x02, // OUT a
            0x50, 0x09, // MVI b <- 9
            0xC4, // XOR a <- b
            0x02, // OUT a
            0x02, // OUT a
            0xE1, 0x01, // APTR 0x00000001
            0x4F, // MV a <- (ptr+c)
            0x02, // OUT a
            0xC1, // CMP
            0x22, 0x1D, 0x00, 0x00, 0x00, // JNZ 0x0000001d
            0x48, 0x30, // MVI a <- 48
            0x02, // OUT a
            0x58, 0x03, // MVI c <- 3
            0x4F, // MV a <- (ptr+c)
            0x02, // OUT a
            0xB0, 0x29, 0x00, 0x00, 0x00, // MVI32 pc <- 0x00000029
            0x48, 0x31, // MVI a <- 49
            0x02, // OUT a
            0x50, 0x0C, // MVI b <- 12
            0xC3, // SUB a <- b
            0x02, // OUT a
            0xAA, // MV32 ptr <- lb
            0x57, // MV b <- (ptr+c)
            0x48, 0x02, // MVI a <- 2
            0xC1, // CMP
            0x21, 0x3A, 0x00, 0x00, 0x00, // JEZ 0x0000003a
            0x48, 0x32, // MVI a <- 50
            0x02, // OUT a
            0x48, 0x77, // MVI a <- 119
            0x02, // OUT a
            0x48, 0x6F, // MVI a <- 111
            0x02, // OUT a
            0x48, 0x72, // MVI a <- 114
            0x02, // OUT a
            0x48, 0x6C, // MVI a <- 108
            0x02, // OUT a
            0x48, 0x64, // MVI a <- 100
            0x02, // OUT a
            0x48, 0x21, // MVI a <- 33
            0x02, // OUT a
            0x01, // HALT
            0x65, 0x6F, 0x33, 0x34, 0x2C, // non-instruction data
        ];

        let output = decode_layer6(program).expect("VM run failed");
        assert_eq!(output, b"Hello, world!");
    }
}

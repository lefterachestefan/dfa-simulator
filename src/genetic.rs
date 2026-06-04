use rkyv::rancor::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use petgraph::graph::DiGraph;
use crate::dfa::Dfa;
use std::collections::{HashMap, VecDeque};

const REG_COUNT: usize = 4;
const MAX_CYCLES: usize = 1000;

#[derive(Clone, Debug, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[repr(u8)]
pub enum Op {
    Add, Sub, Mul, Div, Mod, IfEq, LoadImm, Mov, Nop, Ret, Xor, JmpIf, Pow, And,
    Root, DivFrac, LoadConst, MulFrac, Not, Or, Log, Loop, Geo, Swap,
    LoadImmBig, DigitsSum, R2F, F2R, Addf, Subf, Mulf, Divf, LoadImmf, LoadImmBigf,
    Retf, Sinf, Powf, Movf, Swapf, Geof, CallSolved,
}

#[derive(Clone, Debug, PartialEq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Instruction {
    pub op: Op,
    pub dst: usize,
    pub src: usize,
    pub imm: i32,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = &self.op;
        let d = &self.dst;
        let s = &self.src;
        let imm = &self.imm;
        match self.op {
            Op::LoadImm | Op::LoadImmBig => write!(f, "{op:?} r{d} <- #{imm}"),
            Op::LoadConst => write!(f, "{op:?} r{d} <- #{imm}"),
            Op::Ret | Op::Nop => write!(f, "{op:?}"),
            Op::Mov | Op::Swap => write!(f, "{op:?} r{d}, r{s}"),
            _ => write!(f, "{op:?} r{d} <- r{s}; (#{imm})"),
        }
    }
}

#[derive(Clone, Debug, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct Agent {
    pub code: Vec<Instruction>,
    pub score: f64,
    pub energy: usize,
    pub percent_hit: f64,
}

impl Agent {
    pub fn load_from_file(path: &str) -> Option<Self> {
        let mut file = File::open(path).ok()?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).ok()?;
        let archived = rkyv::access::<ArchivedAgent, Error>(&buf).ok()?;
        rkyv::deserialize::<Agent, Error>(archived).ok()
    }

    pub fn execute(&self, input: i32, prev_agents: &[Agent]) -> (i32, usize) {
        let mut regs = [0i128; REG_COUNT];
        let mut float_regs = [0f64; REG_COUNT];
        regs[0] = input as i128;
        float_regs[0] = input as f64;
        
        let mut counter_reg: i128 = -1;
        let mut pc = 0;
        let mut cycles = 0;
        let len = self.code.len();

        while pc < len && cycles < MAX_CYCLES {
            let instr = &self.code[pc];
            cycles += 1;
            let d = instr.dst % REG_COUNT;
            let s = instr.src % REG_COUNT;

            match instr.op {
                Op::Add => regs[d] = regs[d].wrapping_add(regs[s]),
                Op::Sub => regs[d] = regs[d].wrapping_sub(regs[s]),
                Op::Mul => regs[d] = regs[d].wrapping_mul(regs[s]),
                Op::Div => {
                    let divisor = regs[s];
                    if divisor == 0 { return (regs[0] as i32, cycles); }
                    regs[d] = regs[d].wrapping_div(divisor);
                }
                Op::Mod => {
                    let divisor = instr.imm as i128;
                    if divisor != 0 { regs[d] = regs[d].wrapping_rem(divisor); }
                }
                Op::IfEq => {
                    if regs[d] != regs[s] { pc += 1; }
                }
                Op::LoadImm => regs[d] = instr.imm as i128,
                Op::Mov => regs[d] = regs[s],
                Op::Nop => (),
                Op::Ret => return (regs[0] as i32, cycles),
                Op::Xor => regs[d] ^= regs[s],
                Op::JmpIf => {
                    if regs[d] == instr.imm as i128 {
                        if s == 0 { return (regs[0] as i32, cycles); }
                        pc = (pc as i32 + instr.src as i32).clamp(0, len as i32 - 1) as usize;
                        continue;
                    }
                }
                Op::Pow => regs[d] = regs[d].wrapping_pow(regs[s] as u32),
                Op::And => regs[d] &= regs[s],
                Op::Not => regs[d] = !regs[s],
                Op::Or => regs[d] |= regs[s],
                Op::Root => {
                    let val = regs[d] as u64;
                    let n = regs[s] as u32;
                    if n == 0 { regs[d] = 1; }
                    else if n == 1 { regs[d] = val as i128; }
                    else {
                        let mut res = 1u64;
                        let mut low = 1u64;
                        let mut high = val;
                        while low <= high {
                            let mid = low + (high - low) / 2;
                            if let Some(p) = mid.checked_pow(n) {
                                if p <= val { res = mid; low = mid + 1; }
                                else { high = mid - 1; }
                            } else { high = mid - 1; }
                        }
                        regs[d] = res as i128;
                    }
                }
                Op::Log => {
                    let arg = instr.imm as i128;
                    let val = regs[s];
                    regs[d] = if arg < 2 || val <= 0 { 0 } else { val.ilog(arg) as i128 };
                }
                Op::Loop => {
                    cycles += 20;
                    if counter_reg == -1 {
                        counter_reg = regs[s];
                    }
                    if counter_reg > 0 {
                        counter_reg -= 1;
                        pc = (pc as i32 - d as i32).clamp(0, pc as i32) as usize;
                        continue;
                    }
                }
                Op::Swap => {
                    let tmp = regs[d];
                    regs[d] = regs[s];
                    regs[s] = tmp;
                }
                Op::DigitsSum => {
                    let mut sum = 0;
                    let mut temp = regs[s].abs();
                    let base = if instr.imm.abs() < 2 { 10 } else { instr.imm.abs() as i128 };
                    while temp != 0 {
                        sum += temp % base;
                        temp /= base;
                    }
                    regs[d] = sum;
                }
                Op::LoadImmBig => regs[d] = instr.imm as i128,
                Op::R2F => float_regs[d] = regs[s] as f64,
                Op::F2R => regs[d] = float_regs[s] as i128,
                Op::Addf => float_regs[d] += float_regs[s],
                Op::Subf => float_regs[d] -= float_regs[s],
                Op::Mulf => float_regs[d] *= float_regs[s],
                Op::Divf => float_regs[d] /= float_regs[s],
                Op::LoadImmf => float_regs[d] = instr.imm as f64,
                Op::Retf => return (float_regs[0] as i32, cycles),
                Op::Movf => float_regs[d] = float_regs[s],
                Op::CallSolved => {
                    let idx = instr.imm as usize;
                    if idx < prev_agents.len() {
                        let (res, c) = prev_agents[idx].execute(regs[d] as i32, &prev_agents[..idx]);
                        regs[d] = res as i128;
                        cycles += c;
                    }
                }
                _ => (), 
            }
            pc += 1;
        }
        (regs[0] as i32, cycles)
    }

    pub fn to_dfa(&self, alphabet: &[String], max_states: usize) -> Dfa {
        let mut graph = DiGraph::new();
        let mut state_to_node = HashMap::new();
        let mut final_states = Vec::new();
        let mut queue = VecDeque::new();

        let initial_state_val = 0i32;
        let initial_node = graph.add_node(initial_state_val as u32);
        state_to_node.insert(initial_state_val, initial_node);
        queue.push_back(initial_state_val);

        while let Some(curr_state) = queue.pop_front() {
            let curr_node = state_to_node[&curr_state];

            // Check if final: Agent returns != 0 for input (state << 8 | 0xFF)
            if self.execute((curr_state << 8) | 0xFF, &[]).0 != 0 {
                final_states.push(curr_node.index() as u32);
            }

            if state_to_node.len() >= max_states {
                continue;
            }

            for (idx, symbol) in alphabet.iter().enumerate() {
                let next_state = self.execute((curr_state << 8) | (idx as i32), &[]).0;
                
                let next_node = if let Some(&node) = state_to_node.get(&next_state) {
                    node
                } else if state_to_node.len() < max_states {
                    let node = graph.add_node(next_state as u32);
                    state_to_node.insert(next_state, node);
                    queue.push_back(next_state);
                    node
                } else {
                    // fallback or skip if too many states
                    continue;
                };

                graph.add_edge(curr_node, next_node, symbol.clone());
            }
        }

        Dfa {
            initial_state: 0, 
            final_states,
            graph,
            alphabet: alphabet.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Automaton;

    #[test]
    fn test_simple_agent_to_dfa() {
        // Agent that implements: next_state = (curr_state + symbol) % 2
        // Final if state == 0
        // Input encoding: (state << 8) | symbol
        // For final check: (state << 8) | 0xFF
        let mut code = vec![
            // Extract symbol: r1 = r0 & 0xFF
            Instruction { op: Op::LoadImm, dst: 1, src: 0, imm: 0xFF },
            Instruction { op: Op::And, dst: 1, src: 0, imm: 0 },
            // Extract state: r0 = r0 >> 8
            Instruction { op: Op::LoadImm, dst: 2, src: 0, imm: 8 },
            Instruction { op: Op::Log, dst: 0, src: 0, imm: 2 }, // This is not exactly shift, but Log can be used or I can just assume input is small
        ];
        
        // Actually, let's just make a very simple agent that returns 1 for even symbols and 0 for odd
        code = vec![
            Instruction { op: Op::LoadImm, dst: 1, src: 0, imm: 0xFF },
            Instruction { op: Op::And, dst: 1, src: 0, imm: 0 }, // r1 = symbol
            Instruction { op: Op::LoadImm, dst: 2, src: 0, imm: 2 },
            // If symbol == 0xFF, return 1 (all states final)
            Instruction { op: Op::LoadImm, dst: 3, src: 0, imm: 0xFF },
            Instruction { op: Op::IfEq, dst: 1, src: 3, imm: 0 },
            Instruction { op: Op::Ret, dst: 0, src: 0, imm: 0 }, // Returns r0 (which is (state<<8)|0xFF, so non-zero)
            
            // next_state = symbol % 2
            Instruction { op: Op::Mod, dst: 1, src: 0, imm: 2 },
            Instruction { op: Op::Mov, dst: 0, src: 1, imm: 0 },
            Instruction { op: Op::Ret, dst: 0, src: 0, imm: 0 },
        ];

        let agent = Agent {
            code,
            score: 0.0,
            energy: 0,
            percent_hit: 0.0,
        };

        let alphabet = vec!["0".to_string(), "1".to_string()];
        let dfa = agent.to_dfa(&alphabet, 10);
        
        assert_eq!(dfa.graph.node_count(), 2);
        assert!(dfa.run("0"));
        assert!(dfa.run("10"));
        assert!(dfa.run("110"));
    }
}

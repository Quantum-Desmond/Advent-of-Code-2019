use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::result;

type Result<T> = result::Result<T, Box<dyn Error>>;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn Error>::from(format!($($tt)*))) }
}

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

#[derive(Clone, Copy, Eq, Debug, PartialEq, Hash)]
enum Parameter {
    Position,
    Immediate,
    Relative
}

#[derive(Clone, Eq, Default, Debug, PartialEq, Hash)]
struct Instruction {
    opcode: usize,
    parameters: Vec<Parameter>
}

impl Instruction {
    fn new(number: usize) -> Result<Instruction> {
        let opcode = number % 100;
        let mut digit_list: Vec<_> = (number / 100).to_string().chars().map(|d| d.to_digit(10).unwrap()).collect();
        digit_list.reverse();

        let params_length = match opcode {
            1 => 3,
            2 => 3,
            3 => 1,
            4 => 1,
            5 => 2,
            6 => 2,
            7 => 3,
            8 => 3,
            9 => 1,
            99 => 0,
            x => return err!("{}", format!("Cannot read opcode: {}", x))
        };

        digit_list.resize(params_length, 0);
        let parameters: Result<Vec<Parameter>> = digit_list.into_iter().map(|d| match d {
            0 => Ok(Parameter::Position),
            1 => Ok(Parameter::Immediate),
            2 => Ok(Parameter::Relative),
            x => err!("{}", format!("Cannot read parameter digit: {}", x))
        }).collect();
        let parameters = parameters?;

        Ok(
            Instruction {
                opcode,
                parameters,
            }
        )

    }
}

struct Program {
    memory: Vec<i64>,
    first_input: i64,
    second_input: i64,
    current_input: usize,
    pointer_idx: usize,
    relative_base: i64
}

impl Program {
    fn new(memory: Vec<i64>, first_input: i64, second_input: i64)  -> Program {
        Program {
            memory,
            first_input,
            second_input,
            current_input: 1,
            pointer_idx: 0,
            relative_base: 0
        }
    }

    fn get_input(&mut self) -> Result<i64> {
        let return_value = match self.current_input {
            1 => {
                self.current_input += 1;
                self.first_input
            },
            2 => self.second_input,
            x => return err!("{}", format!("Cannot understand input number {}", x))
        };


        Ok(return_value)
    }

    fn set_input(&mut self, input: i64) {
        self.second_input = input;
    }

    fn get_parameter(&mut self, parameter_form: Parameter, val: i64) -> i64 {
        use self::Parameter::*;

        match parameter_form {
            Position => {
                let idx = val as usize;
                if self.memory.len() < idx+1 {
                    self.memory.resize(idx+1, 0);
                }

                self.memory[idx]
            },
            Immediate => val,
            Relative => {
                let idx = (self.relative_base + val) as usize;
                if self.memory.len() < idx+1 {
                    self.memory.resize(idx+1, 0);
                }

                self.memory[idx]
            }
        }
    }

    fn set_parameter(&mut self, idx: usize, val: i64) -> Result<()> {
        if self.memory.len() < idx+1 {
            self.memory.resize(idx+1, 0);
        }

        self.memory[idx] = val;

        Ok(())
    }

    fn run_program(&mut self) -> Result<Option<i64>> {
        loop {
            let current_instruction = Instruction::new(self.memory[self.pointer_idx] as usize)?;
            match current_instruction.opcode {
                1 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    let output_idx = self.memory[self.pointer_idx+3] as usize;
                    self.set_parameter(output_idx, input_1 + input_2)?;

                    self.pointer_idx += 4;
                },
                2 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    let output_idx = self.memory[self.pointer_idx+3] as usize;
                    self.set_parameter(output_idx, input_1 * input_2)?;

                    self.pointer_idx += 4;
                },
                3 => {
                    let output_idx = self.memory[self.pointer_idx+1] as usize;
                    let input = self.get_input()?;
                    self.set_parameter(output_idx, input)?;

                    self.pointer_idx += 2;
                },
                4 => {
                    let output_idx = self.memory[self.pointer_idx+1];
                    self.pointer_idx += 2;

                    return Ok(Some(self.get_parameter(Parameter::Immediate, output_idx)));
                },
                5 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    if input_1 != 0 {
                        self.pointer_idx = input_2 as usize;
                    } else {
                        self.pointer_idx += 3;
                    }
                },
                6 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    if input_1 == 0 {
                        self.pointer_idx = input_2 as usize;
                    } else {
                        self.pointer_idx += 3;
                    }
                },
                7 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    let output_idx = self.memory[self.pointer_idx+3] as usize;
                    self.set_parameter(output_idx, if input_1 < input_2 {1} else {0})?;

                    self.pointer_idx += 4;
                },
                8 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    let input_2 = self.get_parameter(
                        current_instruction.parameters[1],
                        self.memory[self.pointer_idx+2],
                    );
                    let output_idx = self.memory[self.pointer_idx+3] as usize;
                    self.set_parameter(output_idx, if input_1 == input_2 {1} else {0})?;

                    self.pointer_idx += 4;
                },
                9 => {
                    let input_1 = self.get_parameter(
                        current_instruction.parameters[0],
                        self.memory[self.pointer_idx+1],
                    );
                    self.relative_base += input_1;

                    self.pointer_idx += 2;
                },
                99 => break,
                x => return err!("{}", format!("Incorrect opcode: {}", x))
            }
        }
        Ok(None)
    }
}

pub fn permutations(size: usize) -> Permutations {
    Permutations { idxs: (0..size).collect(), swaps: vec![0; size], i: 0 }
}

pub struct Permutations {
    idxs: Vec<usize>,
    swaps: Vec<usize>,
    i: usize,
}

impl Iterator for Permutations {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i > 0 {
            loop {
                if self.i >= self.swaps.len() { return None; }
                if self.swaps[self.i] < self.i { break; }
                self.swaps[self.i] = 0;
                self.i += 1;
            }
            self.idxs.swap(self.i, (self.i & 1) * self.swaps[self.i]);
            self.swaps[self.i] += 1;
        }
        self.i = 1;
        Some(self.idxs.clone())
    }
}

fn get_permutations(size: usize) -> Vec<Vec<usize>> {
    let perms = Permutations { idxs: (0..size).collect(), swaps: vec![0; size], i: 0 };
    perms.collect::<Vec<_>>()
}

pub fn q1(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let memory: Vec<i64> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q1(memory).unwrap()
}

fn _q1(memory: Vec<i64>) -> Result<usize> {
    let mut program = Program::new(memory, 1, 0);
    let mut last_output = 0;
    while let Some(result) = program.run_program()? {
        last_output = result;
        println!("Result outputted = {}", result);
    }

    Ok(last_output as usize)
}

pub fn q2(fname: String) -> usize {
    let mut f = File::open(fname).expect("File not found");
    let mut f_contents = String::new();

    f.read_to_string(&mut f_contents).expect("Couldn't find file");

    let memory: Vec<i64> = f_contents.trim().split(',').map(|s| s.parse().unwrap()).collect();

    _q2(memory).unwrap()
}

fn _q2(memory: Vec<i64>) -> Result<usize> {
    let amp_count = 5;
    let permutations = get_permutations(amp_count);

    let mut max_signal = 0;
    for permutation in permutations {
        let mut amp_idx = 0;
        let mut output_signal = 0;
        let mut input: i64 = 0;
        let mut Programs: Vec<Program> = permutation.iter().map(|&n| {
            Program::new(memory.clone(), (n + 5) as i64, input)
        }).collect();
        loop {
            let amp = &mut Programs[amp_idx];
            amp.set_input(input);

            if let Some(output_value) = amp.run_program()? {
                input = output_value;
            } else {
                if output_signal > max_signal {
                    max_signal = output_signal;
                }
                break;
            }

            if amp_idx == 4 {
                output_signal = input;
            }
            amp_idx = (amp_idx + 1) % 5;
        }
    }

    Ok(max_signal as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day09_q1_test1() {
        let new_program: Vec<i64> = "104,1125899906842624,99".to_string().split(',').map(|s| s.parse().unwrap()).collect();

        assert_eq!(
            _q1(new_program).unwrap(),
            1125899906842624
        )
    }

    #[test]
    fn day09_q1_test2() {
        let new_program: Vec<i64> = "1102,34915192,34915192,7,4,7,99,0".to_string().split(',').map(|s| s.parse().unwrap()).collect();

        let mut program = Program::new(new_program, 1, 1);
        let mut output = vec![];
        while let Some(result) = program.run_program().unwrap() {
            output.push(result);
        }

        assert!(
            output.iter().any(|n: &i64| (*n).to_string().chars().count() == 16)
        )

    }

    #[test]
    fn day09_q1_test3() {
        let new_program: Vec<i64> = "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99".to_string().split(',').map(|s| s.parse().unwrap()).collect();

        let mut program = Program::new(new_program.clone(), 1, 1);
        let mut output = vec![];
        while let Some(result) = program.run_program().unwrap() {
            output.push(result);
        }

        assert_eq!(
            output,
            new_program
        )
    }
}

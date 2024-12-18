use std::{
    env, fs,
    io::{self, Read},
    process,
};

#[derive(Clone, Debug)]
enum Token {
    /// `>`: Move the instruction pointer to the left (increment)
    IncrementPointer,
    /// `<`: Move the instruction pointer to the right (decrement)
    DecrementPointer,
    /// `+`: Increment the value of the current cell
    Increment,
    /// `-`: Decrement the value of the current cell
    Decrement,
    /// `.`: Output the value of the current cell
    Output,
    /// `,`: Replace the value of the current cell with input
    Input,
    /// `[`: Jump to the matching `]` instruction if the current value is zero
    LoopOpen,
    /// `]`: Jump to the matching `[` instruction if the current value is not zero
    LoopClose,
}

impl TryFrom<char> for Token {
    /// The error is `()` since the result should simply be ignored, since every character that is not a valid one is a comment in brainfuck.
    type Error = ();

    fn try_from(symbol: char) -> Result<Self, Self::Error> {
        match symbol {
            '>' => Ok(Token::IncrementPointer),
            '<' => Ok(Token::DecrementPointer),
            '+' => Ok(Token::Increment),
            '-' => Ok(Token::Decrement),
            '.' => Ok(Token::Output),
            ',' => Ok(Token::Input),
            '[' => Ok(Token::LoopOpen),
            ']' => Ok(Token::LoopClose),
            _ => Err(()),
        }
    }
}

fn lexer(source: impl Into<String>) -> Vec<Token> {
    let mut tokens = vec![];

    for symbol in source.into().chars() {
        // Every other character that is not a valid token is simply ignored
        if let Ok(token) = symbol.try_into() {
            tokens.push(token);
        }
    }

    tokens
}

#[derive(Clone, Debug)]
enum Instruction {
    /// `>`: Move the instruction pointer to the left (increment)
    IncrementPointer,
    /// `<`: Move the instruction pointer to the right (decrement)
    DecrementPointer,
    /// `+`: Increment the value of the current cell
    Increment,
    /// `-`: Decrement the value of the current cell
    Decrement,
    /// `.`: Output the value of the current cell
    Output,
    /// `,`: Replace the value of the current cell with input
    Input,
    /// `[` and `]`: Loop over a vector of instructions
    Loop(Vec<Instruction>),
}

impl TryFrom<Token> for Instruction {
    /// The error is `()` since the result should simply be ignored, since every character that is not a valid one is a comment in brainfuck.
    type Error = ();

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token {
            Token::IncrementPointer => Ok(Instruction::IncrementPointer),
            Token::DecrementPointer => Ok(Instruction::DecrementPointer),
            Token::Increment => Ok(Instruction::Increment),
            Token::Decrement => Ok(Instruction::Decrement),
            Token::Output => Ok(Instruction::Output),
            Token::Input => Ok(Instruction::Input),
            _ => Err(()),
        }
    }
}

fn parser(tokens: Vec<Token>) -> Vec<Instruction> {
    let mut instructions = vec![];

    let mut loop_stack = 0;
    let mut loop_start = 0;

    for (i, token) in tokens.iter().enumerate() {
        if loop_stack == 0 {
            match token {
                Token::LoopOpen => {
                    loop_start = i;
                    loop_stack += 1;
                }
                Token::LoopClose => panic!("loop ending at {i} has no beginning"),
                token => instructions.push(token.clone().try_into().unwrap()),
            }
        } else {
            match token {
                Token::LoopOpen => loop_stack += 1,
                Token::LoopClose => {
                    loop_stack -= 1;
                    if loop_stack == 0 {
                        instructions.push(Instruction::Loop(parser(
                            tokens[loop_start + 1..i].to_vec(),
                        )))
                    }
                }
                _ => (),
            }
        }
    }

    if loop_stack != 0 {
        panic!("loop that starts at {loop_start} has no ending");
    }

    instructions
}

fn run(instructions: &Vec<Instruction>, tape: &mut Vec<u8>, data_pointer: &mut usize) {
    for instruction in instructions {
        match instruction {
            Instruction::IncrementPointer => *data_pointer += 1,
            Instruction::DecrementPointer => *data_pointer -= 1,
            Instruction::Increment => tape[*data_pointer] = tape[*data_pointer].wrapping_add(1),
            Instruction::Decrement => tape[*data_pointer] = tape[*data_pointer].wrapping_sub(1),
            Instruction::Output => print!("{}", tape[*data_pointer] as char),
            Instruction::Input => {
                let mut input: [u8; 1] = [0; 1];
                io::stdin()
                    .read_exact(&mut input)
                    .expect("failed to read stdin");
                tape[*data_pointer] = input[0];
            }
            Instruction::Loop(instructions) => {
                while tape[*data_pointer] != 0 {
                    run(&instructions, tape, data_pointer)
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("usage: {} <input>", &args[0]);
        process::exit(1);
    }

    let path = &args[1];
    let source = fs::read_to_string(path).expect("failed to read source file");

    let tokens = lexer(source);
    let instructions = parser(tokens);

    let mut tape: Vec<u8> = vec![0; 24576];
    let mut data_pointer = 12288;

    run(&instructions, &mut tape, &mut data_pointer);
}

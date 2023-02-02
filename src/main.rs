// Copyright (C) 2023 Petr Pavlu <petr.pavlu@dagobah.cz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(let_chains)]

/// Minimal lexer to read disassembled instructions.
struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

enum Token {
    Word(String),
    Number(i64),
    Comma,
    LeftParenthesis,
    RightParenthesis,
    End,
    Error,
}

impl Lexer {
    /// Creates a new lexer instance.
    pub fn new(instr: &str) -> Self {
        Self {
            chars: instr.chars().collect(),
            pos: 0,
        }
    }

    /// Returns a next token in the instruction string.
    pub fn next(&mut self) -> Token {
        // Skip over any leading whitespace.
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1
        }

        // Check for the end of the string.
        if self.pos >= self.chars.len() || self.chars[self.pos] == '#' {
            return Token::End;
        }

        // Check for and parse a hexadecimal number.
        if self.pos + 1 < self.chars.len()
            && self.chars[self.pos] == '0'
            && (self.chars[self.pos + 1] == 'x' || self.chars[self.pos + 1] == 'X')
        {
            self.pos += 2;
            let mut val = 0u64;
            while self.pos < self.chars.len() && let Some(hex) = self.chars[self.pos].to_digit(16) {
                val = (val << 4) + u64::from(hex);
                self.pos += 1;
            }
            return Token::Number(i64::from_ne_bytes(val.to_ne_bytes()));
        }

        // Check for and parse a decimal number.
        if self.chars[self.pos] == '-' || self.chars[self.pos].is_ascii_digit() {
            let mut neg = false;
            if self.chars[self.pos] == '-' {
                neg = true;
                self.pos += 1;
            }
            let mut val = 0i64;
            while self.pos < self.chars.len() && let Some(dec) = self.chars[self.pos].to_digit(10) {
                val = 10 * val + (if neg { -i64::from(dec) } else { i64::from(dec) });
                self.pos += 1;
            }
            return Token::Number(val);
        }

        // Check for and parse a word.
        let is_word_char = |c: char| -> bool { c.is_ascii_alphanumeric() || c == '.' };
        if is_word_char(self.chars[self.pos]) {
            let start = self.pos;
            while self.pos < self.chars.len() && is_word_char(self.chars[self.pos]) {
                self.pos += 1;
            }
            return Token::Word(self.chars[start..self.pos].iter().collect());
        }

        // Check for and parse special characters.
        if self.chars[self.pos] == ',' {
            self.pos += 1;
            return Token::Comma;
        }
        if self.chars[self.pos] == '(' {
            self.pos += 1;
            return Token::LeftParenthesis;
        }
        if self.chars[self.pos] == ')' {
            self.pos += 1;
            return Token::RightParenthesis;
        }

        Token::Error
    }
}

/// Prepares a given instruction for canonical comparison.
fn canonicalize(instr: &str) -> String {
    let mut res = String::new();
    let mut lexer = Lexer::new(instr);
    loop {
        let token = lexer.next();
        match token {
            Token::Word(word) => {
                let add_space = res.len() == 0;
                res.push_str(&word);
                if add_space {
                    res.push('\t');
                }
            }
            Token::Number(number) => res.push_str(&number.to_string()),
            Token::Comma => res += ", ",
            Token::LeftParenthesis => res += "(",
            Token::RightParenthesis => res += ")",
            Token::End => break,
            Token::Error => {
                eprintln!(
                    "Failed to parse instruction '{}', at position {}",
                    instr, lexer.pos
                );
                break;
            }
        }
    }
    if res.starts_with(".2byte") {
        return "unimp\t".to_string();
    }
    res
}

/// Checks instructions in the 2-byte compressed range.
fn test_2b(gnu: &mut asmct::gnu::GNU, llvm: &mut asmct::llvm::LLVM) -> std::io::Result<()> {
    for byte1 in 0u8..=255 {
        for byte0_h in 0u8..=63 {
            for byte0_l in 0u8..=2 {
                let byte0 = (byte0_h << 2) | byte0_l;
                let bytes = [byte0, byte1];

                let gnu_asm = gnu.disassemble(&bytes);
                let llvm_asm = llvm.disassemble(&bytes);

                let gnu_canon = canonicalize(&gnu_asm);
                let llvm_canon = canonicalize(&llvm_asm);
                println!(
                    "0x{:02x}{:02x}: GNU='{:40}', LLVM='{:40}'{}",
                    byte1,
                    byte0,
                    &gnu_asm.replace("\t", " "),
                    &llvm_asm.replace("\t", " "),
                    if gnu_canon != llvm_canon {
                        ", not matching!"
                    } else {
                        ""
                    }
                );
            }
        }
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    // Initialize both disassemblers.
    let mut gnu = match asmct::gnu::GNU::new() {
        Ok(gnu) => gnu,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };
    let mut llvm = match asmct::llvm::LLVM::new() {
        Ok(llvm) => llvm,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };

    // Check the 2-byte compressed space.
    test_2b(&mut gnu, &mut llvm)?;

    Ok(())
}

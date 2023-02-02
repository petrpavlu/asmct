// Copyright (C) 2023 Petr Pavlu <petr.pavlu@dagobah.cz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![feature(c_variadic)]

pub mod gnu;
pub mod llvm;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl Error {
    fn new(msg: &str) -> Error {
        Error {
            msg: msg.to_string(),
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

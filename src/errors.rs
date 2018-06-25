// Copyright (c) 2018 King's College London
// Created by the Software Development Team <http://soft-dev.org/>
//
// The Universal Permissive License (UPL), Version 1.0
//
// Subject to the condition set forth below, permission is hereby granted to any
// person obtaining a copy of this software, associated documentation and/or
// data (collectively the "Software"), free of charge and under any and all
// copyright rights in the Software, and any and all patent rights owned or
// freely licensable by each licensor hereunder covering either (i) the
// unmodified Software as contributed to or provided by such licensor, or (ii)
// the Larger Works (as defined below), to deal in both
//
// (a) the Software, and
// (b) any piece of software and/or hardware listed in the lrgrwrks.txt file
// if one is included with the Software (each a "Larger Work" to which the Software
// is contributed by such licensors),
//
// without restriction, including without limitation the rights to copy, create
// derivative works of, display, perform, and distribute the Software and make,
// use, sell, offer for sale, import, export, have made, and have sold the
// Software and the Larger Work(s), and to sublicense the foregoing rights on
// either these or other terms.
//
// This license is subject to the following condition: The above copyright
// notice and either this complete permission notice or at a minimum a reference
// to the UPL must be included in all copies or substantial portions of the
// Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::error::Error;
use std::fmt::{self, Formatter, Display};
use std::io;
use elf;

#[derive(Debug)]
pub enum SMParserError {
    /// Parse error from the elf library.
    ElfParse(elf::ParseError),
    /// Generic IO error.
    IO(io::Error),
    /// Other error.
    Other(String),
}

impl Display for SMParserError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SMParserError::ElfParse(e) => write!(f, "{:?}", e), // `e` doesn't implement `Display`.
            SMParserError::IO(e) => Display::fmt(e, f),
            SMParserError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl Error for SMParserError {
    fn description(&self) -> &str {
        match self {
            SMParserError::ElfParse(_) => "ELF parse error",
            SMParserError::IO(e) => e.description(),
            SMParserError::Other(_) => "Other ykstackmaps error",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self {
            SMParserError::ElfParse(_) => None, // Doesn't implement `Error`.
            SMParserError::IO(ref e) => Some(e),
            SMParserError::Other(_) => None,
        }
    }

}

impl From<io::Error> for SMParserError {
    fn from(e: io::Error) -> Self {
        SMParserError::IO(e)
    }
}

impl From<elf::ParseError> for SMParserError {
    fn from(e: elf::ParseError) -> Self {
        SMParserError::ElfParse(e)
    }
}

pub (crate) type SMParserResult<T> = Result<T, SMParserError>;

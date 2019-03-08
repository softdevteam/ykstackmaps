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

// Parse the stackmap section of an ELF binary containing stackmap records.
//
// The comments in this file reference the "Stack Map Format" section of the LLVM documentation
// found here:
// https://llvm.org/docs/StackMaps.html#stack-map-format

extern crate elf;
extern crate byteorder;

mod errors;
#[macro_use]
mod util;

use std::path::Path;
use std::io::Cursor;
use byteorder::{NativeEndian, ReadBytesExt};
use errors::{SMParserError, SMParserResult};
use util::{cursor_skip, cursor_align8, cursor_from_elf};

// We only support this version of the stackmap header for now.
const STACKMAP_VERSION: u8 = 3;

// Sizes in bytes.
const SIZE_STACK_SIZE_ENTRY: u8 = 24;
const SIZE_CONSTANT_ENTRY: u8 = 8;
const SIZE_LOC_ENTRY: u8 = 12;
const SIZE_LIVEOUT_ENTRY: u8 = 4;

/// Offsets into the stackmap section.
const OFFS_STACK_SIZE_ENTRIES: u64 = 16;

/// Represents a single stackmap record entry.
#[derive(Debug, Eq, PartialEq)]
pub struct SMRec {
    id: u64,            // Stackmap ID.
    offset: u32,        // Stackmap offset from start of containing func.
    num_locs: u16,
    locs: Vec<SMLoc>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SMLoc {
    kind: LocKind,
    size: u16,
    dwarf_reg: u16,
    offset: LocOffset,
}

/// Unfortunately, due to a discrepancy between the llvm stackmap documentation
/// [0] and the implementation of their own stackmap parser [1], we need this
/// enum to interpret the integer type differently depending on it's `LocKind`.
/// The offset is interpreted as a u32 *iff* the `LocKind` is a Constant. In all
/// other cases, this is an i32. There are examples [2] in the LLVM test suite
/// where the offset value contains an integer which won't fit inside an i32.
/// For now, we interpret this in the same way that llvm-readobj, and it's test
/// suite expects.
///
/// [0] https://llvm.org/docs/StackMaps.html#id10
/// [1] https://github.com/llvm/llvm-project/blob/57b38a8593bd7d63b9db09676087365d8d3d0d8a/llvm/include/llvm/Object/StackMapParser.h#L123
/// [2] https://github.com/llvm/llvm-project/blob/master/llvm/test/CodeGen/X86/stackmap-large-location-size.ll
///
// #XXX: Update this when we get clarification on what the correct behaviour is
// from the LLVM devs.
#[derive(Debug, Eq, PartialEq)]
pub enum LocOffset {
    I32(i32),
    U32(u32)
}

#[derive(Debug, Eq, PartialEq)]
pub enum LocKind {
    Register,
    Direct,
    Indirect,
    Constant,
    ConstIndex
}

impl LocKind {
    fn from_hex(val: u8) -> SMParserResult<LocKind>{
        match val {
            0x1 => Ok(LocKind::Register),
            0x2 => Ok(LocKind::Direct),
            0x3 => Ok(LocKind::Indirect),
            0x4 => Ok(LocKind::Constant),
            0x5 => Ok(LocKind::ConstIndex),
            _ => Err(SMParserError::Other("Unknown location kind".to_string()))
        }
    }
}

impl SMRec {
    /// Returns the stackmap ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the offset of the stackmap from the start of the containing function.
    pub fn offset(&self) -> u32 {
        self.offset
    }
}

/// Represents a single function entry.
#[derive(Debug, Eq, PartialEq)]
pub struct SMFunc {
    addr: u64,          // Function address.
    stack_size: u64,    // Function's stack size.
}

impl SMFunc {
    /// Get the function address.
    pub fn addr(&self) -> u64 {
        self.addr
    }

    /// Get the size of the stack of the function.
    pub fn stack_size(&self) -> u64 {
        self.stack_size
    }
}

/// An iterator over stackmap record entries.
pub struct SMRecIterator<'a> {
    elf_file: &'a elf::File,
    cursor: Option<Cursor<&'a Vec<u8>>>, // Lazily created so that creating the iterator doesn't
                                         // need to return a `Result`.
    start_pos: u64,                      // Start position of the cursor.
    num_stackmaps: u32,
}

impl<'a> Iterator for SMRecIterator<'a> {
    type Item = SMParserResult<SMRec>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_stackmaps == 0 {
            return None;
        }

        if self.cursor.is_none() {
            self.cursor = Some(itry!(cursor_from_elf(self.elf_file, self.start_pos)));
        }
        let mut cursor = self.cursor.as_mut().unwrap();

        // StkMapRecord[NumRecords] {
        //     uint64: PatchPoint ID
        let id = itry!(cursor.read_u64::<NativeEndian>());
        //     uint32: Instruction Offset
        let offset = itry!(cursor.read_u32::<NativeEndian>());

        // At this point we have everything we need from this entry, but need to skip the remainder
        // of the (variable-sized) entry to find the start of the next.

        //     uint16: Reserved (record flags)
        itry!(cursor_skip(&mut cursor, 2));

        //     uint16: NumLocations
        let num_locs = itry!(cursor.read_u16::<NativeEndian>());
        //     Location[NumLocations] { ... }
        let loc_iter = SMLocIterator {
            elf_file: self.elf_file,
            cursor: None,
            start_pos: cursor.position(),
            num_locs: num_locs
        };

        let mut locs = Vec::with_capacity(num_locs as usize);
        for loc in loc_iter {
            locs.push(loc.expect("malformed location"))
        }

        let skip_locs_sz = u64::from(u32::from(num_locs) * u32::from(SIZE_LOC_ENTRY));
        itry!(cursor_skip(&mut cursor, skip_locs_sz as i64));

        //     uint32: Padding (only if required to align to 8 byte)
        //     uint16: Padding
        itry!(cursor_align8(&mut cursor));
        itry!(cursor_skip(&mut cursor, 2));

        //     uint16: NumLiveOuts
        let num_liveouts = itry!(cursor.read_u16::<NativeEndian>());
        //     LiveOuts[NumLiveOuts] { ... }
        let skip_liveouts_len = i64::from(num_liveouts) * i64::from(SIZE_LIVEOUT_ENTRY);
        itry!(cursor_skip(&mut cursor, skip_liveouts_len));

        //     uint32: Padding (only if required to align to 8 byte)
        itry!(cursor_align8(&mut cursor));
        // } -- End of this stackmap record.

        self.num_stackmaps -= 1;
        Some(Ok(SMRec { id, offset, num_locs, locs }))
    }
}

struct SMLocIterator<'a> {
    elf_file: &'a elf::File,
    cursor: Option<Cursor<&'a Vec<u8>>>,
    start_pos: u64,
    num_locs: u16
}

impl<'a> Iterator for SMLocIterator<'a> {
    type Item = SMParserResult<SMLoc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_locs == 0 {
            return None;
        }

        if self.cursor.is_none() {
            self.cursor = Some(itry!(cursor_from_elf(self.elf_file, self.start_pos)));
        }

        let cursor = self.cursor.as_mut().unwrap();
        // Location[NumRecords] {
        //     uint8: Register | Direct | Indirect | Constant | ConstIndex
        let kind = itry!(cursor.read_u8());
        let kind = itry!(LocKind::from_hex(kind));
        //     uint8: Reserved (expected to be 0)
        let reserved = itry!(cursor.read_u8());
        assert_eq!(reserved, 0);
        //     uint16: Location Size
        let size = itry!(cursor.read_u16::<NativeEndian>());
        //     uint16: Dwarf RegNum
        let dwarf_reg = itry!(cursor.read_u16::<NativeEndian>());
        //     uint16: Reserved (expected to be 0)
        let reserved = itry!(cursor.read_u16::<NativeEndian>());
        assert_eq!(reserved, 0);
        //     iint32 | uint32 : Offset
        let offset = match kind {
            LocKind::Constant => {
                let v = itry!(cursor.read_u32::<NativeEndian>());
                LocOffset::U32(v)
            },
            _ => {
                let v = itry!(cursor.read_i32::<NativeEndian>());
                LocOffset::I32(v)
            }
        };
        self.num_locs -= 1;
        Some(Ok(SMLoc { kind, size, dwarf_reg, offset }))
    }
}

/// An iterator over function entries.
pub struct SMFuncIterator<'a> {
    elf_file: &'a elf::File,
    cursor: Option<Cursor<&'a Vec<u8>>>, // Lazily created so that creating the iterator doesn't
                                         // need to return a `Result`.
    start_pos: u64,                      // Start position of the cursor.
    num_funcs: u32,
}

impl<'a> Iterator for SMFuncIterator<'a> {
    type Item = SMParserResult<SMFunc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num_funcs == 0 {
            return None;
        }

        if self.cursor.is_none() {
            self.cursor = Some(itry!(cursor_from_elf(self.elf_file, self.start_pos)));
        }
        let cursor = self.cursor.as_mut().unwrap();

        // StkSizeRecord[NumFunctions] {
        //     uint64: Function Address
        let addr = itry!(cursor.read_u64::<NativeEndian>());
        //     uint64: Stack Size
        let stack_size = itry!(cursor.read_u64::<NativeEndian>());
        //     uint64: Record Count
        itry!(cursor_skip(cursor, 8));
        // } -- End of this function entry.

        self.num_funcs -= 1;
        Some(Ok(SMFunc{addr, stack_size}))
    }
}

/// Top-level struct through which the user interfaces with the stackmap section.
pub struct StackMapParser {
    elf_file: elf::File,
    num_funcs: u32,
    num_consts: u32,
    num_stackmaps: u32,
}

impl StackMapParser {
    pub fn new(path: &Path) -> SMParserResult<Self> {
        let elf_file = elf::File::open_path(path)?;
        let num_funcs;
        let num_consts;
        let num_stackmaps;
        {
            let mut cursor = cursor_from_elf(&elf_file, 0)?;
            Self::check_header(&mut cursor)?;

            // Read in table sizes.
            // uint32: NumFunctions
            num_funcs = cursor.read_u32::<NativeEndian>()?;
            // uint32: NumConstants
            num_consts = cursor.read_u32::<NativeEndian>()?;
            // uint32: NumRecords
            num_stackmaps = cursor.read_u32::<NativeEndian>()?;
        }

        Ok(Self{elf_file, num_funcs, num_consts, num_stackmaps})
    }

    /// Returns the number of stackmap record entries in the stackmap section.
    pub fn num_stackmaps(&self) -> u32 {
        self.num_stackmaps
    }

    /// Returns the number of function entries in the stackmap section.
    pub fn num_funcs(&self) -> u32 {
        self.num_funcs
    }

    /// Check the stackmap header looks sane.
    fn check_header(cursor: &mut Cursor<&Vec<u8>>) -> SMParserResult<()> {
        // uint8: Stack Map Version
        let version = cursor.read_u8()?;
        if version != STACKMAP_VERSION {
            let msg = format!("Expected stackmap format v{} but binary is v{}", STACKMAP_VERSION, version);
            return Err(SMParserError::Other(msg));
        }
        // uint8: Reserved (expected to be 0)
        let b2 = cursor.read_u8()?;
        if b2 != 0 {
            let msg = format!("Expected 0 in stackmap section byte 2, got {}", b2);
            return Err(SMParserError::Other(msg));
        }
        // uint16: Reserved (expected to be 0)
        let b2_3 = cursor.read_u16::<NativeEndian>()?;
        if b2_3 != 0 {
            let msg = format!("Expected 0 in stackmap section bytes 2 and 3, got {}", b2_3);
            return Err(SMParserError::Other(msg));
        }
        Ok(())
    }

    /// Make an iterator over the stackmap record entries in the stackmap section.
    ///
    /// If the iterator returns an error, the iterator becomes invalid and reuse will lead to
    /// undefined behaviour.
    ///
    /// # Example
    /// ```
    /// use std::path::Path;
    /// use ykstackmaps::StackMapParser;
    ///
    /// match StackMapParser::new(&Path::new("/bin/ls")) {
    ///     // It's unlikey /bin/ls contains stackmaps, but you get the idea.
    ///     Err(e) => println!("error: {}", e),
    ///     Ok(p) =>  {
    ///         for stmap_res in p.iter_stackmaps() {
    ///             match stmap_res {
    ///                 Ok(stmap) => println!("{:?}", stmap),
    ///                 Err(e) => {
    ///                     println!("error: {}", e);
    ///                     break; // You must not re-use the iterator upon error.
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    pub fn iter_stackmaps(&self) -> SMRecIterator {
        let start_pos = OFFS_STACK_SIZE_ENTRIES + u64::from(self.num_funcs) *
            u64::from(SIZE_STACK_SIZE_ENTRY) + u64::from(self.num_consts) *
            u64::from(SIZE_CONSTANT_ENTRY);
        SMRecIterator{
            elf_file: &self.elf_file,
            cursor: None,
            start_pos,
            num_stackmaps: self.num_stackmaps
        }
    }

    /// Make an iterator over functions defined in the stackmap section.
    ///
    /// If the iterator returns an error, the iterator becomes invalid and reuse will lead to
    /// undefined behaviour.
    ///
    /// # Example
    /// ```
    /// use std::path::Path;
    /// use ykstackmaps::StackMapParser;
    ///
    /// match StackMapParser::new(&Path::new("/bin/ls")) {
    ///     // It's unlikey /bin/ls contains stackmaps, but you get the idea.
    ///     Err(e) => println!("error: {}", e),
    ///     Ok(p) =>  {
    ///         for stmap_res in p.iter_functions() {
    ///             match stmap_res {
    ///                 Ok(stmap) => println!("{:?}", stmap),
    ///                 Err(e) => {
    ///                     println!("error: {}", e);
    ///                     break; // You must not re-use the iterator upon error.
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    pub fn iter_functions(&self) -> SMFuncIterator {
        SMFuncIterator{
            elf_file: &self.elf_file,
            cursor: None,
            start_pos: OFFS_STACK_SIZE_ENTRIES,
            num_funcs: self.num_funcs,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::iter::Iterator;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use super::{SMFunc, SMRec, SMLoc, StackMapParser, LocKind, LocOffset};

    #[cfg(target_os="linux")]
    const MAKE: &str = "make";
    const LLVM_READOBJ_PATH: &str = "LLVM_READOBJ_PATH";

    // Invokes GNU make to build a test input.
    fn build_test_inputs(path: &PathBuf) {
        // Change into the `test_inputs` source directory.
        let md = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut dir = PathBuf::from(md);
        dir.push("test_inputs");
        env::set_current_dir(&dir).unwrap();

        // Run make.
        let res = Command::new(MAKE)
                          .arg(path.to_str().unwrap())
                          .output()
                          .unwrap();
        if !res.status.success() {
            eprintln!("build test input failed: \n>>> stdout");
            eprintln!("stdout: {}", String::from_utf8_lossy(&res.stdout));
            eprintln!("\n>>> stderr");
            eprintln!("stderr: {}", String::from_utf8_lossy(&res.stderr));
            panic!();
        }
    }

    // Get the absolute path to the dir containing the test binaries.
    fn test_bin_path(dir: &str, bin: &str) -> PathBuf {
        let md = env::var("CARGO_MANIFEST_DIR").unwrap();
        let mut pb = PathBuf::from(md);
        pb.push("target");
        pb.push("test_inputs");
        pb.push(dir);
        pb.push(bin);
        pb
    }

    // Construct an SMFunc struct from the parsing the output of a Function line
    // in llvm-readobj. Example format:
    //    Function address: 0, stack size: 8, callsite record count: 1
    fn parse_fn(line: &str) -> SMFunc {
        let elems: Vec<&str> = line.split(|c| c == ',' || c == ':').collect();
        // Gives ["Function address", "0", "stack size", "8",  .... ]

        let addr = elems[1].trim().parse::<u64>().unwrap();
        let stack_size = elems[3].trim().parse::<u64>().unwrap();
        SMFunc { addr, stack_size }
    }

    fn parse_loc(line: &str) -> SMLoc {
        let elems: Vec<&str> = line.split(|c| c == ',' || c == ':').collect();
        // Gives ["#N " LocKind +Data", "0", "size", " 8"]

        let size = elems[3].trim().parse::<u16>().unwrap();
        let mut loc = elems[1].trim().split_whitespace();
        let kind = loc.next().unwrap();
        let rest: Vec<&str> = loc.collect();

        match kind {
            "Register" => {
                // e.g rest: ["R#0"]
                let kind = LocKind::Register;
                let dwarf_reg = rest[0].trim_start_matches("R#").parse::<u16>().unwrap();
                let offset = LocOffset::I32(0);
                SMLoc { kind, size, dwarf_reg, offset }
            },
            "Direct" => {
                // e.g rest: ["R#0", "+", "-40"]
                let kind = LocKind::Direct;
                let dwarf_reg = rest[0].trim_start_matches("R#").parse::<u16>().unwrap();
                let offset = LocOffset::I32(rest[2].parse::<i32>().unwrap());
                SMLoc { kind, size, dwarf_reg, offset }
            },
            "Indirect" => {
                // e.g rest: ["[R#0", "+", "-40]"]
                let kind = LocKind::Indirect;
                let dwarf_reg = rest[0].trim_start_matches("[R#").parse::<u16>().unwrap();
                let offset = {
                    let n = rest[2].trim_end_matches("]").parse::<i32>().unwrap();
                    LocOffset::I32(n)
                };
                SMLoc { kind, size, dwarf_reg, offset }
            },
            "Constant" => {
                let kind = LocKind::Constant;
                let dwarf_reg = 0;
                let offset = {
                    let c = rest[0].parse::<u32>().unwrap();
                    LocOffset::U32(c)
                };
                SMLoc { kind, size, dwarf_reg, offset }
            },
            "ConstantIndex" => {
                let kind = LocKind::ConstIndex;
                let dwarf_reg = 0;
                let offset = {
                    let c = rest[0].trim_start_matches("#").parse::<i32>().unwrap();
                    LocOffset::I32(c)
                };
                SMLoc { kind, size, dwarf_reg, offset }
            },
            _ => panic!("Unidentified Location Kind"),
        }
    }

    // Creates an `SMRec` struct from the iterator over lines of strings given.
    // This will increment the iterator past the lines needed parsing.
    fn parse_record<'a, I>(lines: &mut I) -> SMRec
    where
        I: Iterator<Item = &'a str>,
    {
        let line = lines.next().unwrap();
        // Record ID: n, instruction offset: m
        let elems: Vec<&str> = line.split(|c| c == ',' || c == ':').collect();
        // e.g ["Record ID:", " 1", " instruction offset", " 4"]

        let id = elems[1].trim().parse::<u64>().unwrap();
        let offset = elems[3].trim().parse::<u32>().unwrap();

        // Location nums line, e.g:
        //  "4 locations:"
        let num_locs = {
            let line = lines.next().unwrap();
            let n = line.split_whitespace().next().unwrap();
            n.parse::<u16>().unwrap()
        };

        // Individual location line, e.g:
        //  "#1: Register #R0, size: 8"
        let mut locs = Vec::with_capacity(num_locs as usize);
        for _ in 0..num_locs {
            let line = lines.next().unwrap();
            locs.push(parse_loc(line));
        }

        // #TODO Live outs line
        lines.next();

        SMRec { id, offset, num_locs, locs }
    }

    // Parse the output of llvm-readelf to get expected outcomes.
    fn get_expected(path: &Path) -> (Vec<SMFunc>, Vec<SMRec>) {
        let readelf = env::var(LLVM_READOBJ_PATH)
            .expect("Testing requires the LLVM_READOBJ_PATH environment variable to be set");         
        let out = Command::new(readelf)
                          .arg("-stackmap")
                          .arg(path.to_str().unwrap())
                          .output()
                          .expect("failed to run llvm-readelf command");
        assert!(out.status.success());
        let stdout = String::from_utf8(out.stdout).unwrap();

        let mut funcs = Vec::new();
        let mut stkmaps = Vec::new();

        let mut lines = stdout.lines();
        while let Some(line) = lines.next() {
            if line.starts_with("Num Functions:") {
                let fns = {
                    let n = line.split(':').last().unwrap().trim();
                    n.parse::<u32>().unwrap()
                };
                for _ in 0..fns {
                    let line = lines.next().unwrap();
                    funcs.push(parse_fn(line))
                }
            }

            if line.starts_with("Num Records:") {
                let rcs = {
                    let n = line.split(':').last().unwrap().trim();
                    n.parse::<u32>().unwrap()
                };
                for _ in 0..rcs {
                    stkmaps.push(parse_record(&mut lines))
                }
            }
        }
        (funcs, stkmaps)
    }

    fn check_expected_stackmaps(path: PathBuf) {
        build_test_inputs(&path);
        let (expect_funcs, expect_stkmaps) = get_expected(&path);
        let p = StackMapParser::new(&path).unwrap();

        assert_eq!(expect_funcs.len(), p.num_funcs() as usize);
        assert_eq!(expect_stkmaps.len(), p.num_stackmaps() as usize);
        for (got, expect) in p.iter_functions().zip(expect_funcs) {
            assert_eq!(got.unwrap(), expect);
        }

        for (got, expect) in p.iter_stackmaps().zip(expect_stkmaps) {
            assert_eq!(got.unwrap(), expect);
        }
    }

    #[test]
    fn test_hello_world1() {
        check_expected_stackmaps(test_bin_path("hello_world", "hello_world1"));
    }

    #[test]
    fn test_hello_world2() {
        check_expected_stackmaps(test_bin_path("hello_world", "hello_world2"));
    }

    #[test]
    fn test_fannkuch_redux() {
        check_expected_stackmaps(test_bin_path("fannkuch_redux", "fannkuch_redux"));
    }
}

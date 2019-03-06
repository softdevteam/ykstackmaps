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
        let sm_id = itry!(cursor.read_u64::<NativeEndian>());
        //     uint32: Instruction Offset
        let sm_offset = itry!(cursor.read_u32::<NativeEndian>());

        // At this point we have everything we need from this entry, but need to skip the remainder
        // of the (variable-sized) entry to find the start of the next.

        //     uint16: Reserved (record flags)
        itry!(cursor_skip(&mut cursor, 2));

        //     uint16: NumLocations
        let num_locs = itry!(cursor.read_u16::<NativeEndian>());
        //     Location[NumLocations] { ... }
        let skip_locs_sz = u64::from(u32::from(num_locs) * u32::from(SIZE_LOC_ENTRY));
        itry!(cursor_skip(&mut cursor, skip_locs_sz as i64));
        //     uint32: Padding (only if required to align to 8 byte)
        //     uint16: Padding
        itry!(cursor_align8(&mut cursor));

        //     uint16: NumLiveOuts
        let num_liveouts = itry!(cursor.read_u16::<NativeEndian>());
        //     LiveOuts[NumLiveOuts] { ... }
        let skip_liveouts_len = i64::from(num_liveouts) * i64::from(SIZE_LIVEOUT_ENTRY);
        itry!(cursor_skip(&mut cursor, skip_liveouts_len));

        //     uint32: Padding (only if required to align to 8 byte)
        itry!(cursor_align8(&mut cursor));
        // } -- End of this stackmap record.

        self.num_stackmaps -= 1;
        Some(Ok(SMRec{id: sm_id, offset: sm_offset}))
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
    use super::{SMFunc, SMRec, StackMapParser};

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

    // Returns the left and right hand side of a comma-separated key-value pair.
    fn parse_key_val(s: &str) -> (&str, &str) {
        let key_val: Vec<&str> = s.split(":").collect();
        assert_eq!(key_val.len(), 2);
        (key_val[0].trim(), key_val[1].trim())
    }

    // Parse the output of llvm-readelf to get expected outcomes.
    fn get_expected(path: &Path) -> (Vec<SMFunc>, Vec<SMRec>) {
        let readelf = match env::var(LLVM_READOBJ_PATH) {
            Ok(val) => val,
            Err(e) => panic!("No {} environment variable provided", LLVM_READOBJ_PATH),
        };
        let out = Command::new(readelf)
                          .arg("-stackmap")
                          .arg(path.to_str().unwrap())
                          .output()
                          .expect("failed to run llvm-readelf command");
        assert!(out.status.success());
        let stdout = String::from_utf8(out.stdout).unwrap();

        let mut funcs = Vec::new();
        let mut stkmaps = Vec::new();
        for line in stdout.lines() {
            if line.starts_with("  Function address:") {
                let elems = line.split(",");
                let mut addr = None;
                let mut stack_size = None;
                for e in elems {
                    let (key, val) = parse_key_val(e);
                    match key {
                        "Function address" => addr = Some(val.parse::<u64>().unwrap()),
                        "stack size" => stack_size = Some(val.parse::<u64>().unwrap()),
                        _ => (),
                    }
                }
                funcs.push(SMFunc{addr: addr.unwrap(), stack_size: stack_size.unwrap()});
            } else if line.starts_with("  Record ID:") {
                let elems = line.split(",");
                let mut id = None;
                let mut offset = None;
                for e in elems {
                    let (key, val) = parse_key_val(e);
                    match key {
                        "Record ID" => id = Some(val.parse::<u64>().unwrap()),
                        "instruction offset" => offset = Some(val.parse::<u32>().unwrap()),
                        _ => (),
                    }
                }
                stkmaps.push(SMRec{id: id.unwrap(), offset: offset.unwrap()});
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

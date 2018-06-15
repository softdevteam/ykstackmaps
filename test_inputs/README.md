# Test Inputs

This directory contains a collection of LLVM bytecode programs which are used
as test inputs.

## Making a New Test Input

Start with a C program as a basis. You can use any C file, but you probably
don't want anything too large.

Put your C program in a subdirectory. Suppose we start with
`myinput/myinput.c`. First get a '.bc' bitcode file:

```
$ clang -emit-llvm -c myinput.c
```

Next disassemble the bitcode file into a human readable '.ll' file:

```
$ llvm-dis myinput.bc
```

Now edit the resulting `.ll` file and add stackmaps and add the signature of
the stackmap intrinsic:

```
declare void @llvm.experimental.stackmap(i64, i32, ...)
```

And also add some calls to the stackmap intrinsic, e.g.:

```
call void (i64, i32, ...) @llvm.experimental.stackmap(i64 1234, i32 0)
```

Then add your new test to `BINS` in the `GNUmakefile` in this directory. Running
`make` will then generate a binary from your `.ll` file.

The `GNUmakefile` outputs binaries to the Rust `target` directory in the crate
root.

; ModuleID = 'fannkuch_redux.bc'
source_filename = "fannkuch_redux.c"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

declare void @llvm.experimental.stackmap(i64, i32, ...)

%struct._IO_FILE = type { i32, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, %struct._IO_marker*, %struct._IO_FILE*, i32, i32, i64, i16, i8, [1 x i8], i8*, i64, i8*, i8*, i8*, i8*, i64, i32, [20 x i8] }
%struct._IO_marker = type { %struct._IO_marker*, %struct._IO_FILE*, i32 }

@maxflips = global i32 0, align 4
@odd = global i32 0, align 4
@checksum = global i32 0, align 4
@t = common global [16 x i32] zeroinitializer, align 16
@s = common global [16 x i32] zeroinitializer, align 16
@max_n = common global i32 0, align 4
@stderr = external global %struct._IO_FILE*, align 8
@.str = private unnamed_addr constant [18 x i8] c"usage: %s number\0A\00", align 1
@.str.1 = private unnamed_addr constant [29 x i8] c"range: must be 3 <= n <= 12\0A\00", align 1
@.str.2 = private unnamed_addr constant [25 x i8] c"%d\0APfannkuchen(%d) = %d\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define i32 @flip() #0 {
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 0, i32 8)
  %1 = alloca i32, align 4
  %2 = alloca i32*, align 8
  %3 = alloca i32*, align 8
  %4 = alloca i32, align 4
  store i32* getelementptr inbounds ([16 x i32], [16 x i32]* @t, i32 0, i32 0), i32** %2, align 8
  store i32* getelementptr inbounds ([16 x i32], [16 x i32]* @s, i32 0, i32 0), i32** %3, align 8
  %5 = load i32, i32* @max_n, align 4
  store i32 %5, i32* %1, align 4
  br label %6

; <label>:6:                                      ; preds = %10, %0
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 1, i32 8)
  %7 = load i32, i32* %1, align 4
  %8 = add nsw i32 %7, -1
  store i32 %8, i32* %1, align 4
  %9 = icmp ne i32 %7, 0
  br i1 %9, label %10, label %16

; <label>:10:                                     ; preds = %6
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 2, i32 8)
  %11 = load i32*, i32** %3, align 8
  %12 = getelementptr inbounds i32, i32* %11, i32 1
  store i32* %12, i32** %3, align 8
  %13 = load i32, i32* %11, align 4
  %14 = load i32*, i32** %2, align 8
  %15 = getelementptr inbounds i32, i32* %14, i32 1
  store i32* %15, i32** %2, align 8
  store i32 %13, i32* %14, align 4
  br label %6

; <label>:16:                                     ; preds = %6
  store i32 1, i32* %1, align 4
  br label %17

; <label>:17:                                     ; preds = %38, %16
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 3, i32 8)
  store i32* getelementptr inbounds ([16 x i32], [16 x i32]* @t, i32 0, i32 0), i32** %2, align 8
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 4, i32 8)
  %18 = load i32, i32* getelementptr inbounds ([16 x i32], [16 x i32]* @t, i64 0, i64 0), align 16
  %19 = sext i32 %18 to i64
  %20 = getelementptr inbounds i32, i32* getelementptr inbounds ([16 x i32], [16 x i32]* @t, i32 0, i32 0), i64 %19
  store i32* %20, i32** %3, align 8
  br label %21

; <label>:21:                                     ; preds = %25, %17
  %22 = load i32*, i32** %2, align 8
  %23 = load i32*, i32** %3, align 8
  %24 = icmp ult i32* %22, %23
  br i1 %24, label %25, label %35

; <label>:25:                                     ; preds = %21
  %26 = load i32*, i32** %2, align 8
  %27 = load i32, i32* %26, align 4
  store i32 %27, i32* %4, align 4
  %28 = load i32*, i32** %3, align 8
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 6, i32 8)
  %29 = load i32, i32* %28, align 4
  %30 = load i32*, i32** %2, align 8
  %31 = getelementptr inbounds i32, i32* %30, i32 1
  store i32* %31, i32** %2, align 8
  store i32 %29, i32* %30, align 4
  %32 = load i32, i32* %4, align 4
  %33 = load i32*, i32** %3, align 8
  %34 = getelementptr inbounds i32, i32* %33, i32 -1
  store i32* %34, i32** %3, align 8
  store i32 %32, i32* %33, align 4
  br label %21

; <label>:35:                                     ; preds = %21
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 7, i32 8)
  %36 = load i32, i32* %1, align 4
  %37 = add nsw i32 %36, 1
  store i32 %37, i32* %1, align 4
  br label %38

; <label>:38:                                     ; preds = %35
  %39 = load i32, i32* getelementptr inbounds ([16 x i32], [16 x i32]* @t, i64 0, i64 0), align 16
  %40 = sext i32 %39 to i64
  %41 = getelementptr inbounds [16 x i32], [16 x i32]* @t, i64 0, i64 %40
  %42 = load i32, i32* %41, align 4
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 8, i32 8)
  %43 = icmp ne i32 %42, 0
  br i1 %43, label %17, label %44

; <label>:44:                                     ; preds = %38
  %45 = load i32, i32* %1, align 4
  ret i32 %45
}

; Function Attrs: noinline nounwind optnone uwtable
define void @rotate(i32) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  store i32 %0, i32* %2, align 4
  %5 = load i32, i32* getelementptr inbounds ([16 x i32], [16 x i32]* @s, i64 0, i64 0), align 16
  store i32 %5, i32* %3, align 4
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 9, i32 8)
  store i32 1, i32* %4, align 4
  br label %6

; <label>:6:                                      ; preds = %19, %1
  %7 = load i32, i32* %4, align 4
  %8 = load i32, i32* %2, align 4
  %9 = icmp sle i32 %7, %8
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 10, i32 5)
  br i1 %9, label %10, label %22

; <label>:10:                                     ; preds = %6
  %11 = load i32, i32* %4, align 4
  %12 = sext i32 %11 to i64
  %13 = getelementptr inbounds [16 x i32], [16 x i32]* @s, i64 0, i64 %12
  %14 = load i32, i32* %13, align 4
  %15 = load i32, i32* %4, align 4
  %16 = sub nsw i32 %15, 1
  %17 = sext i32 %16 to i64
  %18 = getelementptr inbounds [16 x i32], [16 x i32]* @s, i64 0, i64 %17
  store i32 %14, i32* %18, align 4
  br label %19

; <label>:19:                                     ; preds = %10
  %20 = load i32, i32* %4, align 4
  %21 = add nsw i32 %20, 1
  store i32 %21, i32* %4, align 4
  br label %6

; <label>:22:                                     ; preds = %6
  %23 = load i32, i32* %3, align 4
  %24 = load i32, i32* %2, align 4
  %25 = sext i32 %24 to i64
  %26 = getelementptr inbounds [16 x i32], [16 x i32]* @s, i64 0, i64 %25
  store i32 %23, i32* %26, align 4
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 11, i32 8)
  ret void
}

; Function Attrs: noinline nounwind optnone uwtable
define void @tk(i32) #0 {
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 12, i32 8)
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca [16 x i32], align 16
  store i32 %0, i32* %2, align 4
  store i32 0, i32* %3, align 4
  %6 = bitcast [16 x i32]* %5 to i8*
  call void @llvm.memset.p0i8.i64(i8* %6, i8 0, i64 64, i32 16, i1 false)
  br label %7

; <label>:7:                                      ; preds = %62, %19, %1
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 13, i32 8)
  %8 = load i32, i32* %3, align 4
  %9 = load i32, i32* %2, align 4
  %10 = icmp slt i32 %8, %9
  br i1 %10, label %11, label %63

; <label>:11:                                     ; preds = %7
  %12 = load i32, i32* %3, align 4
  call void @rotate(i32 %12)
  %13 = load i32, i32* %3, align 4
  %14 = sext i32 %13 to i64
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 14, i32 8)
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 15, i32 8)
  %15 = getelementptr inbounds [16 x i32], [16 x i32]* %5, i64 0, i64 %14
  %16 = load i32, i32* %15, align 4
  %17 = load i32, i32* %3, align 4
  %18 = icmp sge i32 %16, %17
  br i1 %18, label %19, label %24

; <label>:19:                                     ; preds = %11
  %20 = load i32, i32* %3, align 4
  %21 = add nsw i32 %20, 1
  store i32 %21, i32* %3, align 4
  %22 = sext i32 %20 to i64
  %23 = getelementptr inbounds [16 x i32], [16 x i32]* %5, i64 0, i64 %22
  store i32 0, i32* %23, align 4
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 16, i32 8)
  br label %7

; <label>:24:                                     ; preds = %11
  %25 = load i32, i32* %3, align 4
  %26 = sext i32 %25 to i64
  %27 = getelementptr inbounds [16 x i32], [16 x i32]* %5, i64 0, i64 %26
  %28 = load i32, i32* %27, align 4
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 17, i32 8)
  %29 = add nsw i32 %28, 1
  store i32 %29, i32* %27, align 4
  store i32 1, i32* %3, align 4
  %30 = load i32, i32* @odd, align 4
  %31 = xor i32 %30, -1
  store i32 %31, i32* @odd, align 4
  %32 = load i32, i32* getelementptr inbounds ([16 x i32], [16 x i32]* @s, i32 0, i32 0), align 16
  %33 = icmp ne i32 %32, 0
  br i1 %33, label %34, label %62

; <label>:34:                                     ; preds = %24
  %35 = load i32, i32* getelementptr inbounds ([16 x i32], [16 x i32]* @s, i64 0, i64 0), align 16
  %36 = sext i32 %35 to i64
  %37 = getelementptr inbounds [16 x i32], [16 x i32]* @s, i64 0, i64 %36
  %38 = load i32, i32* %37, align 4
  %39 = icmp ne i32 %38, 0
  br i1 %39, label %40, label %42

; <label>:40:                                     ; preds = %34
  %41 = call i32 @flip()
  br label %43

; <label>:42:                                     ; preds = %34
  br label %43

; <label>:43:                                     ; preds = %42, %40
  %44 = phi i32 [ %41, %40 ], [ 1, %42 ]
  store i32 %44, i32* %4, align 4
  %45 = load i32, i32* %4, align 4
  %46 = load i32, i32* @maxflips, align 4
  %47 = icmp sgt i32 %45, %46
  br i1 %47, label %48, label %50

; <label>:48:                                     ; preds = %43
  %49 = load i32, i32* %4, align 4
  store i32 %49, i32* @maxflips, align 4
  br label %50

; <label>:50:                                     ; preds = %48, %43
  %51 = load i32, i32* @odd, align 4
  %52 = icmp ne i32 %51, 0
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 0, i32 8)
  br i1 %52, label %53, label %56

; <label>:53:                                     ; preds = %50
  %54 = load i32, i32* %4, align 4
  %55 = sub nsw i32 0, %54
  br label %58

; <label>:56:                                     ; preds = %50
  %57 = load i32, i32* %4, align 4
  br label %58

; <label>:58:                                     ; preds = %56, %53
  %59 = phi i32 [ %55, %53 ], [ %57, %56 ]
  %60 = load i32, i32* @checksum, align 4
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 0, i32 8)
  %61 = add nsw i32 %60, %59
  store i32 %61, i32* @checksum, align 4
  br label %62

; <label>:62:                                     ; preds = %58, %24
  br label %7

; <label>:63:                                     ; preds = %7
  ret void
}

; Function Attrs: argmemonly nounwind
declare void @llvm.memset.p0i8.i64(i8* nocapture writeonly, i8, i64, i32, i1) #1

; Function Attrs: noinline nounwind optnone uwtable
define i32 @main(i32, i8**) #0 {
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 0, i32 8)
  %3 = alloca i32, align 4
  %4 = alloca i32, align 4
  %5 = alloca i8**, align 8
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 0, i32 8)
  %6 = alloca i32, align 4
  store i32 0, i32* %3, align 4
  store i32 %0, i32* %4, align 4
  store i8** %1, i8*** %5, align 8
  %7 = load i32, i32* %4, align 4
  %8 = icmp slt i32 %7, 2
  br i1 %8, label %9, label %15

; <label>:9:                                      ; preds = %2
  %10 = load %struct._IO_FILE*, %struct._IO_FILE** @stderr, align 8
  %11 = load i8**, i8*** %5, align 8
  %12 = getelementptr inbounds i8*, i8** %11, i64 0
  %13 = load i8*, i8** %12, align 8
  %14 = call i32 (%struct._IO_FILE*, i8*, ...) @fprintf(%struct._IO_FILE* %10, i8* getelementptr inbounds ([18 x i8], [18 x i8]* @.str, i32 0, i32 0), i8* %13)
  call void @exit(i32 1) #5
  unreachable

; <label>:15:                                     ; preds = %2
  %16 = load i8**, i8*** %5, align 8
  %17 = getelementptr inbounds i8*, i8** %16, i64 1
  %18 = load i8*, i8** %17, align 8
  %19 = call i32 @atoi(i8* %18) #6
  store i32 %19, i32* @max_n, align 4
  %20 = load i32, i32* @max_n, align 4
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 0, i32 8)
  %21 = icmp slt i32 %20, 3
  br i1 %21, label %25, label %22

; <label>:22:                                     ; preds = %15
  %23 = load i32, i32* @max_n, align 4
  %24 = icmp sgt i32 %23, 15
  br i1 %24, label %25, label %28

; <label>:25:                                     ; preds = %22, %15
  %26 = load %struct._IO_FILE*, %struct._IO_FILE** @stderr, align 8
  %27 = call i32 (%struct._IO_FILE*, i8*, ...) @fprintf(%struct._IO_FILE* %26, i8* getelementptr inbounds ([29 x i8], [29 x i8]* @.str.1, i32 0, i32 0))
  call void @exit(i32 1) #5
  unreachable

; <label>:28:                                     ; preds = %22
  store i32 0, i32* %6, align 4
  br label %29

; <label>:29:                                     ; preds = %38, %28
  %30 = load i32, i32* %6, align 4
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 0, i32 8)
  %31 = load i32, i32* @max_n, align 4
  %32 = icmp slt i32 %30, %31
  br i1 %32, label %33, label %41

; <label>:33:                                     ; preds = %29
  %34 = load i32, i32* %6, align 4
  %35 = load i32, i32* %6, align 4
  %36 = sext i32 %35 to i64
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 0, i32 8)
  %37 = getelementptr inbounds [16 x i32], [16 x i32]* @s, i64 0, i64 %36
  store i32 %34, i32* %37, align 4
  br label %38

; <label>:38:                                     ; preds = %33
  %39 = load i32, i32* %6, align 4
  %40 = add nsw i32 %39, 1
  store i32 %40, i32* %6, align 4
  br label %29

; <label>:41:                                     ; preds = %29
  %42 = load i32, i32* @max_n, align 4
  call void @tk(i32 %42)
  %43 = load i32, i32* @checksum, align 4
  %44 = load i32, i32* @max_n, align 4
  %45 = load i32, i32* @maxflips, align 4
  %46 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([25 x i8], [25 x i8]* @.str.2, i32 0, i32 0), i32 %43, i32 %44, i32 %45)
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 0, i32 8)
  call void (i64, i32, ...) @llvm.experimental.stackmap(i64 0, i32 8)
  ret i32 0
}

declare i32 @fprintf(%struct._IO_FILE*, i8*, ...) #2

; Function Attrs: noreturn nounwind
declare void @exit(i32) #3

; Function Attrs: nounwind readonly
declare i32 @atoi(i8*) #4

declare i32 @printf(i8*, ...) #2

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { argmemonly nounwind }
attributes #2 = { "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #3 = { noreturn nounwind "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #4 = { nounwind readonly "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #5 = { noreturn nounwind }
attributes #6 = { nounwind readonly }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 6.0.1-svn333623-1~exp1~20180604223356.84 (branches/release_60)"}

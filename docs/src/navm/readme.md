# navm 虚拟机标准

本次大作业的编译目标是 Natrium 虚拟机 (navm) 的汇编 (s0)。其设计参考了 JVM、DotNet CLR 和上学期的 c0 虚拟机。

## 虚拟机简介

navm 是一个 [栈式虚拟机][stack_machine] —— 简单来说就是，它的寄存器是一个栈。除了少数内存访问指令以外，navm 的大部分指令都只操作位于栈顶的数据。堆栈式计算机的指令与 [逆波兰表示法（后缀表示法][reverse_polish_notation] 表示的表达式（或者说后序遍历的表达式树）有简单的对应关系。

navm 有 64 位有符号整数、无符号整数、浮点数三种数据类型。详见 [数据类型](#数据类型) 节。

navm 使用 64 位的地址空间，详见 [地址空间](#地址空间) 节。

navm 使用自制的指令集，共有 50+ 个指令，详见 [指令集](#指令集) 节。

[stack_machine]: https://en.wikipedia.org/wiki/Stack_machine
[reverse_polish_notation]: https://en.wikipedia.org/wiki/Reverse_Polish_notation

## 内存空间

navm 的内存空间以 8 位（1 字节）为单位寻址。8、16、32、64 位的数据类型分别以其大小为单位在内存中对齐。当读取或写入操作未对齐时，会产生 `UnalignedAccess` 错误。

navm 的栈空间以 8 字节为一个 slot，压栈、弹栈以及各种运算操作均以 slot 为单位进行。默认情况下，栈的大小是 1 MiB (1048576 字节)，即 131072 个 slot。栈空时弹栈和栈满时压栈分别会产生 `StackUnderflow` 和 `StackOverflow` 错误。

## 数据类型

navm 在运算中支持三种基本数据类型，分别是 64 位无符号整数 `u64`、64 位有符号整数 `i64`、64 位浮点数 `f64`。长度更短的整数可以使用 `u64` 和 `i64` 模拟。

`u64` 和 `i64` 都是 64 位整数，使用[二进制补码][2s_complement]形式表示。两种类型在多数整数运算中不做区分，仅在 `cmp.T`（比较指令，见下）等两种运算结果有差别的地方有所区分。在运算溢出时，两种类型均采用环绕 (wrap-around) 方式处理结果。`u64` 同时也可以表示虚拟机中的内存地址。

`f64` 是符合 [IEEE 754][ieee754] 规定的[双精度浮点数][double]。

[2s_complement]: https://en.wikipedia.org/wiki/Two%27s_complement
[ieee754]: https://en.wikipedia.org/wiki/IEEE_754
[double]: https://en.wikipedia.org/wiki/Double-precision_floating-point_format

## 二进制格式

s0 是 navm 所使用的汇编文件格式，其作用和内容类似 Java 的 `.class` 文件或者 DotNet 的 `.dll` 文件。

下面的结构体表示了 s0 的二进制文件结构。其中，`uXX` 表示 XX 位无符号整数 。

```
/// 整个 S0 二进制文件
struct S0 {
    /// 魔数
    magic: u32 = 0x72303b3e,
    /// 版本号，定为 1
    version: u32 = 0x00000001,
    /// 标志位，留空
    flags: u32,
    /// 全局变量表
    globals: Array<GlobalDef>,
    /// 函数列表
    functions: Array<FunctionDef>,
}

/// 数组
struct Array<T> {
    /// 数组的长度
    count: u32,
    /// 数组的元素，无间隔排列
    items: T[],
}

/// 单个全局变量
struct GlobalDef {
    /// 是否为常量？非零值视为真
    is_const: u8,
    /// 值
    value: Array<u8>,
}

/// 函数
struct FunctionDef {
    /// 函数名称在全局变量中的位置
    name: u16,
    /// 返回值占据的 slot 数
    return_slots: u16,
    /// 参数占据的 slot 数
    param_slots: u16,
    /// 局部变量占据的 slot 数
    loc_slots: u16,
    /// 函数体
    body: Array<Instruction>,
}

/// 指令
union Instruction {
    /// 无参数的指令
    variant NoParam {
        opcode: u8
    },
    /// 有 4 字节参数的指令
    variant u32Param {
        opcode: u8,
        param: u32,
    }
    /// 有 8 字节参数的指令
    variant u64Param {
        opcode: u8,
        param: u64
    }
}
```

## 栈帧结构

> 这里描述的是 **这个** navm 实现中使用的栈帧结构。其他实现可能与本实现具有不同的栈帧结构，和/或使用不同的方式传递参数、储存局部变量等。

```
| ...           |
|               | <- 栈顶 %sp
| 表达式栈 ...  |
| 表达式栈      |
| 局部变量 ...  |
| 局部变量      |
| 虚拟机参数... | 
| 虚拟机参数    | <- 被调用者栈底 %bp 
|===============|===
| 调用参数 ...  | v
| 调用参数      | |
| 返回值        | |
| 中间结果      | 调用者栈
| ...           | ^ 
|===============|===
```

其中，调用参数和返回值由调用者压栈，调用参数在函数返回后由被调用者清理。

### 虚拟机参数

虚拟机会在调用参数和局部变量之间插入一系列的虚拟机参数以辅助虚拟机运行，目前本虚拟机存储的参数格式为（从栈顶到栈底）：

```
| ...             |
| 局部变量        |
|=================|
| 调用者函数 ID   |
| 调用者 %ip      |
| 调用者 %bp      |
|=================|
| 参数            |
| ...             |
```

### 函数调用时栈帧变化示例

假设现有一个函数 `test`，有 1 slot 的返回值、2 slot 的参数和 2 slot 的局部变量。

```rust,ignore
/// 可以看成是这样的一个函数
fn test(a: int, b: int) -> int {
    let c: int = ...;
    let d: int = ...;
    ...
    return ...;
}
```

现在，它被编号为 1 的函数 `main` 调用，则执行 `call` 指令前的栈如下（不规定参数压栈顺序）：

```
| -          |
|============|<- 栈顶
| b          | ↑
| a          | 参数
| _ret       | 返回值
| ...        | ...表达式栈
```

在执行 `call` 指令后，栈如下：

```
| -            | <- 栈顶（表达式栈）
| d            | ↑
| c            | 局部变量   
|==============|
| 1            | ↑          
| %ip          |            
| %bp          | 虚拟机数据 
|==============|
| b            | ↑          
| a            | 参数       
| _ret         | 返回值     
| ...          |
```

在函数调用返回后，栈如下：

```
| -          | 
| // d       |  
| // c       |
| // 1       | 
| // %ip     | 
| // %bp     |  ↑
| // b       |  |
| // a       | 以上内容被弹栈
|============| <- 栈顶
| _ret       | 返回值
| ...        |
```

## 程序入口

navm 总是会最先运行函数列表里编号为 0 的（也就是整个列表中第一个）函数，按照惯例这个函数的名称为 `_start`。`_start` 函数没有任何参数，也不返回任何值，这两项的参数会被忽略。`_start` 函数不能有返回指令。

一般来说，程序会在 `_start` 中设置全局变量的值，以及进行其他的准备工作。在准备工作完成之后，`_start` 函数应当调用 `main` 函数开始正式的程序运行。如果需要，`_start` 函数也可以在 `main` 函数返回之后进行清理工作。`_start` 函数不需要返回。

一个示例的 `_start` 函数如下：

```
fn _start 0 0 -> 0 {
    // global(1) = 1 + 1;
    globa 1
    push 1
    push 1
    add.i
    store.64
    // main();
    call 4
    // cleanup: global(1) = 0;
    globa 1
    push 0
    store.64
    // no return
}
```

# NewScript Interpreter

This is the repository for the NewScript interpreter built with Rust. NewScript is a high-order dynamically typed programming language that uses a register-based virtual machine. The NewScript interpreter can be used via a CLI to run scripts or a REPL interface, and also can be embedded into Rust programs to be used to create a programmable interface.

> [!WARNING]  
> This project and its artifacts are purely for academic purposes and are not suitable for professional use. Use with caution, as it may contain bugs, security vulnerabilities, or undefined behaviour.


## Installation and Setup

To build the project and (optionally) install the NewScript interpreter onto your system, or embed NewScript as a library, use one of the following guides:

### Requirements:
+ `rustc` version `>=1.85.0`
+ `cargo` version `>=1.85.0`

### Local build and run:
1. Download the source code from the zip or from [github.com/ChandruSu/nscript](https://github.com/ChandruSu/nscript) (repository will not be public until after examination)

2. Open a shell session in the directory and execute the following commands:
    ```sh
    cargo build --release
    ```
3. Run cli and execute sample script like so:
    ```sh
    target/release/ns help

    target/release/ns run <path/to/script.ns>

    # example
    target/release/ns run ./examples/mandel.ns
    ```

### System install and run:
1. Download the source code from the zip or from [github.com/ChandruSu/nscript](https://github.com/ChandruSu/nscript)

2. Open a shell session in the directory and execute the following commands:
    ```sh
    cargo install --path .
    ```
3. Run cli and execute sample script like so:
    ```sh
    ns help

    ns run <path/to/script.ns>

    # example
    ns run ./examples/mandel.ns
    ```

### Use as Library (Cargo)

To embed the NewScript interpreter into your Rust project with the `cargo` package manager, you can:
1. Download the source code from the zip or from [github.com/ChandruSu/nscript](https://github.com/ChandruSu/nscript) and include the whole directory in your project

2. Then add the path to the `nscript` project root directory to the `Cargo.toml` file of your project, under dependencies:
    ```toml
    [dependencies]
    ns = { path = "path/to/nscript/" }
    ```
  
3. Then you can use the interpreter like so:
    ```rust
    use ns::Interpreter;

    fn main() {
        let mut nsi = Interpreter::new(false, false, vec![]);
        let _ = nsi.execute_from_string(
          "import(\"std\").println(\"Hello World!\");"
        );
    }
    ```

### Benchmarking

To view guide on benchmarks, see [benchmarks/README.md](./benchmarks/README.md), so that you can perform simple benchmarks on your own machine.

## User Guide

### Command Line Interface (CLI)

To use the CLI tool, you can run `ns help` to print a guide of all the subcommands to the terminal. Here is a full detailing of current options:

#### Subcommands

Command|Description|Arguments
:---|:---|:---
`help`|Prints help dialogue and lists subcommands|N/a|
`run`|Executes NewScript from specified file|FILEPATH|
`eval`|Executes NewScript expression from string argument|EXPR|
`repl`|Start a REPL session|N/a|

#### Options

Option|Short|Description|Arguments
:---|:---|:---|:---
`--debug`|`-d`|Runs in debug mode/Display AST+Bytecode|N/a
`--verbose`|`-v`|Runs in verbose mode/Display phase times|N/a
`--args`|`-a`|Pass arguments to program|ARG LIST

You can run a NewScript program with command line arguments, like so:

```sh
# Runs mandelbrot set at .6 scale
ns run ./examples/mandel.ns --args 0.6
```

You can start an interactive REPL session like so:
```sh
ns repl
```

### Crate Library

If you choose to embed the NewScript interpreter into your Rust application, here is how you can use the crate with your project. Ensure to follow the setup guide to see how to link the crate from the source code.

Import required exports from `ns` crate.
```rust
use ns::{Interpreter, ModuleFnRecord, NativeFnPtr, Value};
```

Create an instance of the interpreter like so (instance must be mutable):
```rust
let mut nsi = Interpreter::new(false, false, Vec::new());
```

You can execute NewScript source code directly from a string like so:
```rust
if let Err(e) = nsi.execute_from_string("let x = 5;") {
    e.dump_error(nsi.environment());
    return;
}
```

Note this will update the environment so that variable and function declarations will persist. You can access the (virtual machine) environment thought `nsi.environment()` or `nsi.environment_mut()`

All executions and evaluations return a `Result<_, ns::error::Error>` which provides can be unwrapped to access to result of an evaluation or used to observe the error encountered.

You can also evaluate expressions and retrieve the result (and run scripts from files with `.execute_from_file("/path/to/file.ns")`). All objects and values in NewScript are of type `ns::Value`

```rust
let result = nsi.evaluate_from_string("x + 2");
if let Err(e) = result {
    e.dump_error(nsi.environment());
    return;
}

assert_eq!(result.unwrap(), Value::Int(7));
```

If you wish to create a programmable interface for your Rust app, allowing users to use NewScript to interact with your program, you can register your own modules which act as an interface for your code:

```rust
let sqr: NativeFnPtr = |env, reg0, _argc| env.reg(reg0) * env.reg(reg0);

nsi.environment_mut().register_module(
    "math".to_string(),
    vec![
        ModuleFnRecord::new("sqr".to_string(), 1, sqr),
        // ...    
    ],
);

let _ = nsi.execute_from_string("let math = import(\"math\");");


let result2 = nsi.evaluate_from_string("math.sqr(3)").unwrap();
assert_eq!(result2, Value::Int(9));
```
Modules will still need to be imported into the environment (including the standard library)

A module declaration is just a list of functions, that adhere to the `NativeFnPtr` type, grouped into an object; modules cannot have global space variables.

Here is how a `NativeFnPtr` method declaration is structured and can be used:

```rust
fn f(
    env: &mut ns::Env, // To access VM environment
    arg0: usize,       // Register location of argument 1
    _argc: usize,      // Number of arguments passed
) -> Result<ns::Value, ns::error::Error> {
    // Access register stack
    let a: &ns::Value = env.reg(arg0);
    let b: &ns::Value = env.reg(arg0 + 1);

    match (a, b) {
        (ns::Value::Int(a), ns::Value::Int(b)) => {
            if a > b {
                Ok(ns::Value::String(Rc::new("Greater".to_string())))
            } else {
                Ok(ns::Value::String(Rc::new("Less Than or Equal".to_string())))
            }
        }
        _ => ns::error::Error::custom_error("Wrong")
            .with_pos(env.last_call_pos()) // To enable stack trace
            .err(),
    }
}
```

> To generate and view a full reference for all the structs and methods exposed by the `ns` crate, you can run `cargo doc` and access to generated [documentation](./target/doc/ns/index.html) (open in external browser)

## Language Reference

1. Variable declarations and assignment;
    ```
    let age = 20;
    let height = 1.8;
    let isMale = true;
    let name = "James";
    let addr = {"line1": "30 Aldwych", "line2": "Strand, London"};
    let aliases = ["Jimmy", "Jimmy"];

    age += 1;
    aliases[1] = "Jimbo";
    ```

2. Single line comments
    ```
    # this is a comment
    ```

3. Imports and Modules
    ```
    let std = import("std");
    std.println("Hello, " + name);
    ```

4. Conditional branching i.e. `if-else` blocks
    ```
    if age > 21 {
       std.println("You're older than 21");
    } else if age < 21  {
        std.println("You're younger than 21");
    } else {
        std.println("You must be 21");
    }
    ```

5. Iteration with `while` loops:
    ```
    let i = std.len(aliases) - 1;
    while i >= 0 {
        std.println(aliases[i]);
        i -= 1;
    }
    ```

6. Function declaration and invocation:
    ```
    fun example(a, b) {
        return a + b;
    }

    std.println(example(1, 2) == 3);
    ```

7. Arithmetic, Boolean, Logical, Bitwise, Ternary and Lambda expressions:

    ```
    let a = 1 * 2 + 4 / 0.5;
    
    let b = (1 > 2) || (3 == 3 && 4 <= 5);

    let c = (24 >> 2) & (3 | 7) ^ 4;

    let x = if 1 > 2 { "A" } else { "B" };

    let y = fun(a, b) { return a + b; };
    ```

8. Closures and high-order functions:
    ```
    fun multiplier(n) {
        return fun (x) {
            return n * x;
        };
    }

    fun map(arr, f) {
        let out = [];
        let i = 0;
        let N = std.len(arr);
        while i < N {
            std.append(out, f(arr[i]));
            i += 1;
        }
        return out;
    }

    std.println(multiplier(2)(33));
    std.println(map([1, 2, 3], multiplier(4)));
    std.println(map([1, 2, 3], fun(x) { return x*x; }));
    ```

9. Arrays and Objects
    ```
    let data = ["a", false, 5.0];
    std.println(data[2]);
    data[1] = "b";

    let addr = {"line1": "30 Aldwych", "line2": "Strand, London"};
    std.println(addr["line1"]);
    std.println(addr.line2);
    addr["postcode"] = "WC2B 4BG";
    ```

    Keys in `{}` objects can be accessed via subscript `[]` or if key is string, with attribute `object.key`

### Standard library

Here are all the methods exposed by the standard library that can be imported via the name `std`, and their descriptions

Method|Description|Arguments|Returns
:---|:---|:---|:---
`print`|Prints value without newline|Any|Null
`println`|Prints value with newline return|Any|Null
`typeof`|Return string name of value's type|Any|String
`len`|Return length of value|String,Object,Array|Int
`str`|Return string form of value|Any|String
`append`|Add value to array|Array,Any|Null
`insert`|Add element to array or object at index/key|Array/Object,Any,Any|Null
`remove`|Remove and return element from array or object by index/key|Array/Object,Any|Any
`pop`|Remove last element from array|Array|Any
`keys`|Return array of Object keys|Object|Array
`gc`|Run garbage collector|None|Null
`time`|Get the current time in milliseconds|None|Int
`parseInt`|Convert String into Int|String|Int
`parseFloat`|Convert String into Float|String|Float


## Copyright and Licencing

The NewScript interpreter will be available under the MIT license, see the [LICENSE](./LICENSE) document for more information.
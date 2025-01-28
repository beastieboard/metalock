
# METALOCK

Metalock is a barebones language designed to be called inside a Solana program.

## But why would you do this?

The use case for Metalock is when you have a smart contract system and you
want to support custom user logic; the overhead for allowing each user to 
specify their own 3rd party contract and call that contract is "too damned high";
the CPI is likely to cost 5000+ CU, as well as require additional annoying
logic to specify their 3rd party program and accounts when creating the TX.

However, a Metalock program which compiles into a few hundred bytes can be easily
be stored inside a user account (which is loaded anyway), and execution costs
can be relatively low.

## Creating a Program

Programs are created using a DSL in Rust.

A program is a function that takes an input and returns an output,
and it compiles into a binary that can be executed later.

In Metalock program code, every value is an `RR`, and every function returns an `RR`. The resulting value is then compiled (at compile time) into an expression which is evaluated at runtime.

For example, here is a program that checks that a number is not 10:

```rust
fn not_10(n: RR<u32>) -> RR<bool> {
    n.equals(10).not()
}
```

To compile this to bytes:

```rust
let code: Vec<u8> = not_10.to_program().compile();
```

To run the program:

```rust
let mut eval = Evaluator::new(&mut code.as_ref(), Default::default());
let r = eval.run(1u32.into())._as::<bool>();

// r is true
```

## Execution model

* The RR value in the DSL represents an expression tree, which is compiled to a simple bytecode.
* As such, the execution model is lazy rather than imperative, except when explicitly sequenced using `.seq(...)`.

## Bytecode

The program not_10 compiles into the bytecode:

```
(06                        // NOT
  (23                      // EQ
    (60 0000)              // VAR lookup of input data + 2 bytes Var ID (0)
    (07 0001 03 0A000000)  // VAL, schema len, schema (u32), u32 little endian
  )
)
```

For a total of 13 bytes (could be further reduced).

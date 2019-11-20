# ViMIB 
###### *VIrtual Machine Inspired by Boredom*

Simple language compiled into bytecode.

## Why?
This language is kind of useless right now.  Even if it is usable in the future,
this language isn't designed to improve upon any other languages so there really
isn't any point in ever using it for real, but it was fun to make.

In other words, **just for fun**

## Bytecode
Currently this language only has the types
 * `i32`
 * `bool`
and a register pool limited to 256 bytes b/c references are only one byte long.

Jumps accept a byte as their input so programs can at max jump to index 255.

In the bytecode all numbers are stored in big-endian format.

Example bytecode (each byte separated by space):
```assembly
0:   push_i   0 0 0 0        ; push 0 to stack
5:   sto_i    0              ; store at 0
7:   load_i   0              ; load from 0
9:   virtual  0              ; print integer at top of stack
11:  load_i   0              ; load from 0
13:  push_i   0 0 0 10       ; push 10 to stack
18:  ge                      ; >= op (returns bool (u8))
19:  if_f     30             ; if false jump to 30
21:  push_i   73 143 58 148  ; push 1234123412 to the stack
26:  virtual  0
28:  goto     43
30:  load_i   0
32:  push_i   0 0 0 1
37:  add_i                   ; add top two integers
38:  sto_i    0
40:  goto     7
```

## Language

That bytecode is generated from the following:
```rust
let i = 0

loop {
    println(i)
    if i >= 10 {
        println(1234123412)
        break
    }
    i = i + 1
}
```
The parser can parse function declarations as well, but there is no bytecode generated yet for functions.  Nested loops also don't generate properly.

## ToDo
 - [ ] strings
 - [ ] floats
 - [ ] 64 bit numbers
 - [ ] better loops
 - [ ] functions
 - [ ] classes
 - [ ] scope
 - [ ] static type checks
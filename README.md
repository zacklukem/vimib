# ViMIB 
###### *VIrtual Machine Inspired by Boredom*
![ci]

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
0:   push_i   0 0 0 0   ; push 0 onto the stack
5:   sto_i    0         ; store in register #0
7:   load_i   0         ; load register #0
9:   virtual  0         ; print int from stack
11:  load_i   0
13:  push_i   0 0 0 10
18:  ge                 ; >= operator
19:  if_f     31        ; if false goto 31
21:  ldc      0         ; load constant #0
23:  virtual  2         ; print string
25:  ldc      7
27:  virtual  2
29:  goto     44
31:  load_i   0
33:  push_i   0 0 0 1
38:  add_i              ; add top two integers
39:  sto_i    0
41:  goto     7
```

## Language

That bytecode is generated from the following:
```rust
let i = 0

loop {
    print_int(i)
    if i >= 10 {
        print_str("Hello,")
        print_str("World!")
        break
    }
    i = i + 1
}
```
The parser can parse function declarations as well, but there is no bytecode generated yet for functions.  Nested loops also don't generate properly.

## ToDo
 - [x] strings
 - [ ] floats
 - [ ] 64 bit numbers
 - [ ] better loops
 - [x] functions
 - [ ] classes
 - [ ] scope
 - [ ] static type checks

[ci]: https://github.com/zacklukem/vimib/workflows/Continuous%20Integration/badge.svg

## Specification
### Numbers Strings Identifiers
```ebnf
digit   = "0" | ... | "9" ;
letter  = "a" | ... | "z" | "A" | ... | "Z" ;
number  = digit, { digit }, [ ".", digit, { digit } ] ;
string  = '"', UTF_8_CHAR_NOT_QUOTE, '"' ;
ident   = ( letter | "_" ), { letter | number | "_" } ;
```

### Expressions
```ebnf
expr    = literal
        | binary
        | unary
        | group
        | call ;

literal = number | string ;
call    = ident, "(", [ expr, { ",", expr } ], ")" ;
binary  = expr, binop, expr ;
unary   = ("!" | "-"), expr ;
group   = "(", expr, ")" ;
binop   = "+" | "-" | "*" | "/" | "%" | "==" 
        | "!=" | "<" | ">" | "<=" | ">="
        | "&&" | "||" | "&" | "|" ;
```

### Statements
```ebnf
stmt    = expr
        | if stmt
        | "loop", block
        | "return", [ expr ]
        | "break"
        | "let", ident, "=", expr
        | ident, "=", expr ;
block   = "{", { stmt }, "}" ;
if stmt = "if", expr, block,
          { "else if", expr, block },
          [ "else", block ] ;
```

### Functions
```ebnf
fn_decl = "fn", ident, "(" [ ident, { ",", ident } ] ")", block ;
```

### Program
```ebnf
program = [ fn_decl ] ;
```
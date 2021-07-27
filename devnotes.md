- [Devnotes](#devnotes)
  - [1. Code Optimization Ideas](#1-code-optimization-ideas)
    - [Order:](#order)
    - [Pattern Match](#pattern-match)
    - [Dead Code](#dead-code)
    - [While unroll](#while-unroll)
    - [Add Successive](#add-successive)
  - [2. Performance improvements](#2-performance-improvements)

# Devnotes
Notes I use during development

## 1. Code Optimization Ideas

### Order: 

pattern_match -> ~~dead_code -> while_unroll~~ -> add_successive

### Pattern Match

```[-]``` == ```*mp = 0``` -> encoded as 'a' for example

### Dead Code

While loops that are never entered can be removed. Same problem as while unroll: how do we know what is or isn't run?

### While unroll

Figure out how many times a loop will run -> Do I have to run it up to that position?? Look into how compilers do it

I don't think so, think we can do it statically. ~~Count backwords until we reach '<' or '>', cound all + or - along the way?~~ No that doesnt work

Only count '>' and '<' and figure out **which** cell we use, **then** count all increases/decreases for that cell?

-> is that faster than just running the damn thing?

Other idea: Run the program, when reaching a while loop check current *mp and copy that while loop that many times -> How much do we need to copy? Need to find bracket first... ugh


### Add Successive

opcode bytes: 

```
+ 00101011
- 00101101
< 00111100
> 00111110

. 00101110
, 00101100
[ 01011011
] 01011101
```

-> Can use the first byte as marker for control bytes:

```
     CAO
x+ = 111XXXXX
x- = 110XXXXX
x< = 101XXXXX
x> = 100XXXXX
```

``C = Control, A = Arithmetic, O = Option``

-> Leaves us 5 bits of storage for "times", starting with 0 = 2 that means:

```
11100000 = add(2)
11100101 = add(7)
11111111 = add (33)
```

We can get the last 5 bits with a fast bitmask: ``X= 111XXXXX & 00011111``

We could also say that if ``add(33) = 11111110 + '+'`` then ``11111111`` means we treat the next 8 bits as one number to be added, if that's also ``11111111`` then we also consider the next 8 bits etc.

## 2. Performance improvements

I realized there is no reason to use bimap, we can just use one Hashmap and add both direction, or, even better:

-> We could save half the space in the hashmap if we don't track the ``'[' -> ']'`` direction and re-implement a runtime-search (since it's an edge-case that ``*mp == 0`` when entering a while loop)

-> Consider/Try a BTreeMap or Changing the Hash Algo of Hashmap, try:
- [FxHash (rustc-hash)](https://lib.rs/crates/rustc-hash)
- [ahash]([ahash](https://lib.rs/crates/ahash))



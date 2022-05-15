
# Design Considirations

1. Since I didn't find it necessary to support unicode, the interpreter supports UTF-8 only which is better performance wise (fast indexing, smaller size, etc).
2. Every program should define `MAIN` function as an entry point.
3. Function calls should be done by `CALL` instruction and all of them should properly return by using `RETURN` or `RETURN_VALUE`. This is necessary because these instructions properly handle the stack and also the frame.
4. Since memory cell is 4-bytes long, instead of pushing characters one by one and using 4-bytes for 1-byte characters, I implemented `PUSH_STR` and `POP_STR` instructions to fit 4 characters in a memory cell.
5. All tokens, instructions, etc. uses `&str` instead of `String`. Because using `String` would result in lots of unnecessary copies.

# Traverse Directory
Run traverse directory (question #4) by running:
```sh
cargo r --example runner -- examples/traverse_dir.bci
```

# Factorial
```sh
cargo r --example runner -- examples/factorial.bci
```

# Custom programs
You can write a bci program and pass it to runner like above to run it.

# Testing

```sh
cargo test
```

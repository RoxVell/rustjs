# RustJs

Simple js engine implemented in Rust

# Examples

Run evaluation using virtual machine, measure time for scanning, parsing, evaluation
```
cargo run -- vm --filename <your_file> --time
```

Run evaluation using ast interpreter, measure time for scanning, parsing, evaluation
```
cargo run -- ast --filename <your_file> --time
```

# TODO

## Javascript features

### Common
- [x] Const variables
- [ ] Array objects

### Class
- [ ] Class inheritance
- [ ] Private fields & methods

### Global ideas
- [ ] Turn into CLI tool
- [ ] Bytecode generator
- [ ] Bytecode interpreter
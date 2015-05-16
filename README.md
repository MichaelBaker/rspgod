# How I got this thing to work

```
>> rustc --version
rustc 1.0.0 (a59de37e9 2015-05-13) (built 2015-05-14)
```

* This compiles the rust program into a dynamic library that can be linked into a c program at runtime. `rustc rust.rs --crate-type=dylib`

* Once you've got your rust .dylib file, you just compile the c program normally. All of the loading is handled at runtime. `gcc c.c -o main`

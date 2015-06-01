# r(u)s(t)p(ost)g(res)o(utput)d(ecoder)

Versions

```
>> rustc --version
rustc 1.0.0 (a59de37e9 2015-05-13) (built 2015-05-14)
```

* To get the C header files into Rust, (edit and then) run `script/import`
* To build the project, run `script/plugin`
* You must use `no_mangle`, `pub`, and `extern` in order for the rust function to be callable from C.

# r(u)s(t)p(ost)g(res)o(utput)d(ecoder)

Versions

```
>> rustc --version
rustc 1.0.0 (a59de37e9 2015-05-13) (built 2015-05-14)
```

To get the C header files into Rust, we compiled rust-bindgen, and then ran this command, and then edited it... or something:

```
env DYLD_LIBRARY_PATH=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/ \
    target/debug/bindgen \
    -o ~/deleteme/rspgod/src/postgres.rs \
    ~/deleteme/rspgod/src/headers.h \
    -I /usr/local/Cellar/postgresql/9.4.1/include/server/ \
    -builtins
```

* You must use no_mangle, pub, and extern in order for the rust function to be callable from C.
* `plugin` is the script that we use to compile and test the Postgres Output Decoder

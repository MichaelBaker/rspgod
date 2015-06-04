# r(u)s(t)p(ost)g(res)o(utput)d(ecoder)

Versions
--------

```
>> rustc --version
rustc 1.0.0 (a59de37e9 2015-05-13) (built 2015-05-14)

>> postgres --version
postgres (PostgreSQL) 9.4.1
```

Building
--------

* You'll need an executable for [rust-bindgen](https://crates.io/crates/rust-bindgen).
  We got ours by building it from source in some other directory.
* You need to set three environment variables:
  * `BINDGEN` Path to the bindgen executable.
  * `DYLD_LIBRARY_PATH` Path to something or other, used by bindgen. On OSX, it's going to be: `/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib`
  * `PG_INCLUDE_DIR` Path to the Postgresql header files that we compile against.

  ```sh
  export DYLD_LIBRARY_PATH=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib
  export PG_INCLUDE_DIR=/usr/local/Cellar/postgresql/9.4.1/include/server/
  export BINDGEN=/Users/josh/deleteme/rust-bindgen/target/debug/bindgen
  ```
* Then you can generate the Postgres library bindings with `script/import`
* Then you can build the project with `script/build`


Notes to self
-------------

* You must use `no_mangle`, `pub`, and `extern` in order for the rust function to be callable from C.

Test Cases
----------

* Tables without a primary key
* One column for every data type

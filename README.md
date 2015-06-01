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

* You'll need to clone rust-bindgen and build it, then edit script/import with the relevant directories.
* Then you can build the project with `script/plugin`


Notes to self
-------------

* You must use `no_mangle`, `pub`, and `extern` in order for the rust function to be callable from C.
* Buy milk

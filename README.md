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
* You will also need a ruby interpreter installed
* Then you can generate the Postgres library bindings with `script/import`
* Then you can build the project with `script/build`

Testing
-------
* You need to set one environment variable `POSTGRES_URL`. This should be a postgres connection string to a database that you've already created for testing purposes. e.g. `postgres://michaeltbaker@localhost:5432/test`
* Then you can run the automated tests with script/test

Notes to self
-------------

* You must use `no_mangle`, `pub`, and `extern` in order for the rust function to be callable from C.
* To print out the JSON of an update for inspection, use

  ```rust
  use rustc_serialize::json::{as_pretty_json};
  println!("{}", as_pretty_json(&fetch_updates(c)));
  ```

Todo
----

* Add primary key to tuples
    * Use `get_primary_key_relation` to retrieve the primary key columns
* Figure out some scheme for error handling
* Use the Postgres include dir in the Makefile
* Automatically source a `.env` file in build and test scripts if it is present. This should make configuration of this project easier.
* Create tests for all replica identity modes
  * REPLICA_IDENTITY_INDEX
  * REPLICA_IDENTITY_FULL
  * REPLICA_IDENTITY_NOTHING
  * REPLICA_IDENTITY_DEFAULT
* Test what happens when `if (repl_ident_oid != InvalidOid)` is false
* Make more fine-grained/safer implementation of `get_primary_key_relation`

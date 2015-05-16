MODULE_big = thingy
OBJS = plugin.o librust_plugin.a

plugin.o: rust_plugin.o plugin.c
	gcc plugin.c -c -o plugin.o -I /usr/local/Cellar/postgresql/9.4.1/include/server -I /usr/local/Cellar/postgresql/9.4.1/include/internal

rust_plugin.o: rust_plugin.rs
	rustc rust_plugin.rs --crate-type=staticlib

PG_CONFIG = pg_config
PGXS := $(shell $(PG_CONFIG) --pgxs)
include $(PGXS)

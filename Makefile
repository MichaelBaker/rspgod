MODULE_big = thingy
OBJS = plugin.o target/debug/librspgod.a

plugin.o: src/plugin.c target/debug/librspgod.a
	gcc src/plugin.c -c -o plugin.o -I /usr/local/Cellar/postgresql/9.4.1/include/server -I /usr/local/Cellar/postgresql/9.4.1/include/internal

target/debug/librspgod.a: src/lib.rs src/postgres.rs
	cargo build

PG_CONFIG = pg_config
PGXS := $(shell $(PG_CONFIG) --pgxs)
include $(PGXS)

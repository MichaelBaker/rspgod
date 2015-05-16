MODULE_big = thingy
OBJS = plugin.o

plugin.o: plugin.c
	gcc plugin.c -c -o plugin.o -I /usr/local/Cellar/postgresql/9.4.1/include/server -I /usr/local/Cellar/postgresql/9.4.1/include/internal

all: plugin.o

PG_CONFIG = pg_config
PGXS := $(shell $(PG_CONFIG) --pgxs)
include $(PGXS)

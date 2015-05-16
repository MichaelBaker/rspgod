MODULE_big = thingy
OBJS = plugin.o plugin_two.o

plugin.o: plugin_two.o plugin.c
	gcc plugin.c -c -o plugin.o -I /usr/local/Cellar/postgresql/9.4.1/include/server -I /usr/local/Cellar/postgresql/9.4.1/include/internal

plugin_two.o: plugin_two.c
	gcc plugin_two.c -c -o plugin_two.o -I /usr/local/Cellar/postgresql/9.4.1/include/server -I /usr/local/Cellar/postgresql/9.4.1/include/internal

PG_CONFIG = pg_config
PGXS := $(shell $(PG_CONFIG) --pgxs)
include $(PGXS)

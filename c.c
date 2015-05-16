#include <stdio.h>
#include <dlfcn.h>
#include <stdint.h>
#include "c.h"

int main() {
  void *hw = dlopen("/Users/michaeltbaker/Projects/rust-ffi-test/librust.dylib", RTLD_NOW);

  int32_t (*add_two)(int32_t);
  add_two = dlsym(hw, "add_two");

  int32_t value = 5;
  printf("add_two: %p\n", add_two);
  printf("Rust sez: %d\n", add_two(5));

  DATA data = { 5, 1.3 };
  printf("\n");
  printf("Before\n");
  printf("Int: %d\n",   data.field_one);
  printf("Float: %f\n", data.field_two);

  void (*rs_print_struct)(*DATA);
  rs_print_struct = dlsym(hw, "rs_print_struct");
  rs_print_struct(&data);

  printf("\n");
  printf("After\n");
  printf("Int: %d\n",   data.field_one);
  printf("Float: %f\n", data.field_two);

  dlclose(hw);
}

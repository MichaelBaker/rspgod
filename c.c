#include <stdio.h>
#include <dlfcn.h>
#include <stdint.h>


int main() {
  void *hw = dlopen("/Users/michaeltbaker/Projects/rust-ffi-test/librust.dylib", RTLD_NOW);
  int32_t (*add_two)(int32_t);
  add_two = dlsym(hw, "add_two");
  int32_t value = 5;
  printf("add_two: %p\n", add_two);
  printf("Rust sez: %d\n", add_two(5));
  dlclose(hw);
}

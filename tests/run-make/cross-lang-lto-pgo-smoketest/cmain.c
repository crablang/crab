#include <stdint.h>

// A trivial function defined in CrabLang, returning a constant value. This should
// always be inlined.
uint32_t crablang_always_inlined();


uint32_t crablang_never_inlined();

int main(int argc, char** argv) {
    return (crablang_never_inlined() + crablang_always_inlined()) * 0;
}

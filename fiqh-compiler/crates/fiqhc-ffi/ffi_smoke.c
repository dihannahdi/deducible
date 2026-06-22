/* Native FFI harness — a C program that embeds the fiqhc engine via the cdylib, exactly as a
 * legacy core-banking system (Java/JNI, C#/P-Invoke, C) would. Reads a .fiqh spec, calls
 * fiqh_check_json, prints the JSON verdict.
 *   gcc ffi_smoke.c -L ../../target/release -lfiqhc_ffi -o ffi_smoke
 *   LD_LIBRARY_PATH=../../target/release ./ffi_smoke
 */
#include <stdio.h>
#include <stdlib.h>

extern char *fiqh_check_json(const unsigned char *p, unsigned long len);
extern void fiqh_free_cstr(char *p);

int main(void) {
    FILE *f = fopen("specs/riba_disguised.fiqh", "rb");
    if (!f) { fprintf(stderr, "cannot open spec\n"); return 1; }
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    unsigned char *buf = (unsigned char *)malloc(n);
    if (fread(buf, 1, n, f) != (size_t)n) { fprintf(stderr, "read error\n"); return 1; }
    fclose(f);

    char *json = fiqh_check_json(buf, (unsigned long)n);
    printf("%s\n", json ? json : "(null)");
    fiqh_free_cstr(json);
    free(buf);
    return 0;
}

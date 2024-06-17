
typedef void (*callback) (void *data);

void baz(callback cb, void *data) {
  cb(data);
}

void bar(callback cb, void *data) {
  baz(cb, data);
}

void foo(callback cb, void *data) {
  bar(cb, data);
}

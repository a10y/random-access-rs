

Results on M2 Max Macbook Pro as of February 2024:

```
Timer precision: 41 ns
random_access  fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ bitpacking  749.6 ns      │ 3.301 µs      │ 791.4 ns      │ 887.9 ns      │ 1000    │ 4000
╰─ snappy      101.6 µs      │ 145.7 µs      │ 106.2 µs      │ 107.7 µs      │ 500     │ 500
```

Bit-unpacking roughly ~100x faster than snappy decompression for random access.


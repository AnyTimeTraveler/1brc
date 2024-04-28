
 # The One Billion Row Challenge

This is my implementation for [The One Billion Row Challenge](https://github.com/gunnarmorling/1brc).

The goal is, to process 1 billion lines of CSV as fast as possible.

# Log

## 27.04.2024

After an afternoon/evening of casual tinkering and swapping out some map implementations for better ones,
I am now at 3.926 seconds.

I wish I had recorded my earlier times.

I do remember that replacing the `std::collections::HashMap` with `halfbrown::HashMap` roughly halved my time.  
But I suspect that there is more performance to be gained from an array-based approach.

I know of several improvements that I could still make:

 - do some profiling to see where the bottlenecks lie
 - take advantage of bitwise operations to avoid branches
 - write my own optimized hash-map replacement
 - write my own float-parser
 - get away from float-parsing entirely and just use an integer

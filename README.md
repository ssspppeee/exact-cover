# Exact cover - Dancing links

A Rust implementation of the "dancing links" algorithm for the exact cover problem as described in Volume 4B of Donald Knuth's The Art of Computer Programming, applied to Sudoku.

Input is a Sudoku formatted as a flattened string with zeroes representing blank spaces.

Output is a similarly formatted string with zeroes replaced by the solved values.

For example:

Input:  

```209000600040870012800019040030700801065008030100030007000650709604000020080301450```

Output: 

```219543678543876912876219345432765891765198234198432567321654789654987123987321456```

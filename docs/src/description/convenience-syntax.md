# Convenience syntax

To make writing constraints easier, AirScript provides a number of syntactic conveniences. These are described in this section.

## List comprehension

List comprehension provides a simple way to create new vectors. It is similar to the list comprehension syntax in Python. The following examples show how to use list comprehension in AirScript.

```
let x = [a * 2 for a in b]
```
This will create a new vector with the same length as `b` and the value of each element will be twice that of the corresponding element in `b`.

```
let x = [a + b for (a, b) in (c, d)]
```
This will create a new vector with the same length as `c` and `d` and the value of each element will be the sum of the corresponding elements in `c` and `d`. This will throw an error if `c` and `d` vectors are of unequal lengths.

```
let x = [2^i * a for (i, a) in (0..5, b)]
```
Ranges can also be used as iterables, which makes it easy to refer to an element and its index at the same time. This will create a new vector with length 5 and each element will be the corresponding element in `b` multiplied by 2 raised to the power of the element's index. This will throw an error if `b` is not of length 5.

```
let x = [m + n + o for (m, n, o) in (a, 0..5, c[0..5])]
```
Slices can also be used as iterables. This will create a new vector with length 5 and each element will be the sum of the corresponding elements in `a`, the range 0 to 5, and the first 5 elements of `c`. This will throw an error if `a` is not of length 5 or if c is of length less than 5.

## List folding

List folding provides syntactic convenience for folding vectors into expressions. It is similar to the list folding syntax in Python. List folding can be applied to vectors, list comprehension or identifiers referring to vectors and list comprehension. The following examples show how to use list folding in AirScript.

```
trace_columns:
    main: [a[5], b, c]

integrity_constraints:
    let x = sum(a)
    let y = sum([a[0], a[1], a[2], a[3], a[4]])
    let z = sum([a * 2 for a in a])
```

In the above, `x` and `y` both represent the sum of all trace column values in the trace column binding `a`. `z` represents the sum of all trace column values in the trace column binding `a` multiplied by `2`.

```
trace_columns:
    main: [a[5], b, c]

integrity_constraints:
    let x = prod(a)
    let y = prod([a[0], a[1], a[2], a[3], a[4]])
    let z = prod([a + 2 for a in a])
```

In the above, `x` and `y` both represent the product of all trace column values in the trace column binding `a`. `z` represents the product of all trace column values in the trace column binding `a` added by `2`.

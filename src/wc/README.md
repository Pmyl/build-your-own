## Benchmarks

## -l -c
`wc` is blazing fast compared to `myown wc`
### myown wc -l -c
| Executed in | 1.84 secs |          fish |  external |
|:------------|----------:|--------------:|----------:|
| usr time    | 1.26 secs |  37.85 millis | 1.22 secs |
| sys time    | 0.48 secs | 239.43 millis | 0.25 secs |

### wc -l -c
| Executed in | 949.25 millis |          fish |      external |
|:------------|--------------:|--------------:|--------------:|
| usr time    | 606.43 millis |  20.01 millis | 586.42 millis |
| sys time    | 308.57 millis | 205.37 millis | 103.20 millis |

## -w
`myown wc` is faster, probably because it's a naive implementation that does
not handle any edge case
### myown wc -w
| Executed in | 1.84 secs |          fish |  external |
|:------------|----------:|--------------:|----------:|
| usr time    | 1.26 secs |  40.41 millis | 1.22 secs |
| sys time    | 0.48 secs | 239.60 millis | 0.24 secs |

### wc -w
| Executed in | 2.08 secs |          fish |  external |
|:------------|----------:|--------------:|----------:|
| usr time    | 1.66 secs |  18.80 millis | 1.64 secs |
| sys time    | 0.41 secs | 213.35 millis | 0.20 secs |

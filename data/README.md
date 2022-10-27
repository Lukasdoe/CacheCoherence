
# Data sets
The `head -n 5` is shown for each file in the zip archive.

`/single_thread/read_miss.zip`:
| Core 0    |
| ----      |
0 0x817ae8

`/single_thread/read_hit.zip`:
| Core 0    |
| ----      |
0 0x817ae8
0 0x817ae8

`/single_thread/write_miss.zip`:
| Core 0    |
| ----      |
1 0x817ae8

`/single_thread/write_hit.zip`:
| Core 0    |
| ----      |
1 0x817ae8
1 0x817ae8

`/single_thread/evict.zip`:
| Core 0    |
| ----      |
0 0x05
0 0x09
0 0x05

`/single_thread/sequence.zip`:
| Core 0    |
| ----      |
0 0x10
2 0xa
1 0x14
0 0x10
1 0x10


`blackscholes_10.zip` |
`blackscholes_10_000.zip` |
`blackscholes_100_000.zip` |
`blackscholes_1000_000.zip`:
| Core 0    | Core 2    | Core 2    | Core 3    |
| ----      | ----      | ----      | ----      |
|0 0x817ae8 | 1 0x7f0a3b28 | 0 0x7fe89980 | 1 0x7f0d3b28 |
|2 0x1b | 2 0x10 | 2 0x2d | 2 0xa |
|0 0x817af8 | 1 0x7f0a3b30 | 0 0x7fc890d4 | 1 0x7f0d3b30 |
|2 0x1f | 2 0x13 | 2 0x29 | 2 0xf |
|0 0x817b08 | 1 0x7f0a3b38 | 0 0x7fc890d4 | 1 0x7f0d3b38 |



















# Additional Assumptions

- Instruction scheduling happens instantly => the clk cycle that a "other" is scheduled is already
  the first cycle in which it is reduced.
- Writing always takes one cycle to hit the cache.
  - Write Hit => 1 cycle delay
  - Write Miss => 1 + (cache_miss_penalty) delay

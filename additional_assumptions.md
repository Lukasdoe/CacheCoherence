# Additional Assumptions

- Instruction scheduling happens instantly => the clk cycle that a "other" is scheduled is already
  the first cycle in which it is reduced.
- Writing always takes one cycle to hit the cache.
  - Write Hit => 1 cycle delay
  - Write Miss => 1 + (cache_miss_penalty) delay
- PrWriteMiss does not exist at the protocol level. Every write miss first allocates using a read.
  Therefore, every PrWriteMiss is translated to PrRdMiss -> Write
- A bus update always only transmits a single word, bus flushes always transmit a whole block.
  Flushes always also go to main memory and therefore require at least 100 cycles. Updates can go to
  other caches only, making them faster with about 2 cycles.

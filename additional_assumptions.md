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
- Dragon protocol: Bus flushes are only required for write backs. As long as the copy stays in the
  cache, every "flush" in the diagram is replaced with a shared update action. No data is written to
  main memory.
- Cache write-allocate: 
  every write checks if the address is currently in cache. If not, it schedules a load and restarts
  the check afterwards. Only if the check is successful (hit), the write is attempted. If another
  cache invalidates the cache line between the read and the write, then the read has to be repeated.
- A write to a cache line does not update its LRU-counter
- Bus wait cycles are only counted beginning in the clock cycle AFTER the task was put on the bus.
  This means that a writeback requires in total 101 cycles until the next action can be performed: 1
  cycle to schedule the writeback and 100 cycles for the bus to finish the writeback to memory.
- Caches block during their own bus transactions => the cache waits until its bus transaction is
  finished until it commences with further steps (this is required to restart the transaction in
  case something fails)
- Other caches may listen to the bus during flushes to main memory and can therefore directly update
  / read their new value. This means that a bus read that causes a flush (MESI) only takes the time
  that is required to flush to main memory (which is > than shared read time).
- Our MESI is Illinois MESI.

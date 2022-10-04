type CacheState = {
  hit: number;
  miss: number;
  tag: number;
  index: number;
  block_offset: number;
  alu: number;
};

export default CacheState;

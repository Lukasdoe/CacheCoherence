export type CacheState = {
  core_id: number;
  cnt: number;
};

export type CacheAccess = {
  core_id: number;

  hit_or_miss: boolean;
  tag: number;
  index: number;
};

export type CacheUpdate = {
  core_id: number;

  old_tag: number;
  new_tag: number;
  index: number;
  block: number;
};

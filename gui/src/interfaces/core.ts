import Record from "./record";
import CacheState from "./cache";

type CoreState = {
  id: number;
  alu: number;
  record: Record;
  cache: CacheState;
};

export default CoreState;

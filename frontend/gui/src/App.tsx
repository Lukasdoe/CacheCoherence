import Header from "./components/header";
import Core from "./components/core";
import Bus from "./components/bus";
import { useState } from "react";
import { CoreState, loadSets } from "./components/core";
import { EnvInfo, Step } from "./interfaces/system";
import { CoreState as APICoreState } from "./interfaces/core";
import { CacheAccess, CacheState, CacheUpdate } from "./interfaces/cache";

const App = () => {
  const [cores, setCores] = useState<CoreState[]>([]);

  const processLoad = ([type, data]: [any, any]) => {
    if (type === "EnvInfo") {
      const parsed_data = data as EnvInfo;
      setCores(
        Array(parsed_data.num_cores)
          .fill(0)
          .map((_, i) => {
            return {
              system: {
                protocol: parsed_data.protocol,
                cache_size: parsed_data.cache_size,
                associativity: parsed_data.associativity,
                block_size: parsed_data.block_size,
                num_cores: parsed_data.num_cores,
                clk: 0,
              },
              id: i,
              alu_cnt: 0,
              cache: {
                cnt: 0,
                hit: false,
                miss: false,
                tag: 0,
                index: 0,
                inv: loadSets(
                  parsed_data.cache_size,
                  parsed_data.associativity,
                  parsed_data.block_size
                ),
              },
              record: "",
            };
          })
      );
    }
  };

  const processData = ([type, data]: [any, any]) => {
    if (type === "Step") {
      const parsed_data = data as Step;
      setCores(
        cores.map((c, i) => {
          return {
            ...c,
            system: {
              ...c.system,
              clk: parsed_data.clk,
            },
          };
        })
      );
    }
    if (type === "CoreInit") {
      // skip
    }
    if (type === "CoreState") {
      const parsed_data = data as APICoreState;
      setCores(
        cores.map((c, i) => {
          return i == parsed_data.id
            ? {
                ...c,
                record: parsed_data.record,
                alu_cnt: parsed_data.alu_cnt,
              }
            : c;
        })
      );
    }
    if (type === "CacheState") {
      const parsed_data = data as CacheState;
      setCores(
        cores.map((c, i) => {
          return i == parsed_data.core_id
            ? {
                ...c,
                cache: {
                  ...c.cache,
                  cnt: parsed_data.cnt,
                },
              }
            : c;
        })
      );
    }
    if (type === "CacheAccess") {
      const parsed_data = data as CacheAccess;
      setCores(
        cores.map((c, i) => {
          return i == parsed_data.core_id
            ? {
                ...c,
                cache: {
                  ...c.cache,
                  hit: parsed_data.hit_or_miss,
                  miss: !parsed_data.hit_or_miss,
                  tag: parsed_data.tag,
                  index: parsed_data.index,
                },
              }
            : c;
        })
      );
    }
    if (type === "CacheUpdate") {
      const parsed_data = data as CacheUpdate;
      setCores(
        cores.map((c, i) => {
          if (i == parsed_data.core_id) {
            c.cache.inv[parsed_data.index][parsed_data.block] =
              parsed_data.new_tag;
          }
          return c;
        })
      );
    }
  };

  const next = () => {
    fetch("http://127.0.0.1:8080/next", { method: "GET" })
      .then((response) => response.json())
      .then(processData);
  };

  const load = () => {
    fetch("http://127.0.0.1:8080/load", { method: "GET" })
      .then((response) => response.json())
      .then(processLoad);
  };

  return (
    <div className="app">
      <Header load={load} next={next} />
      <div className="cores">
        {cores.map((core: CoreState) => (
          <Core key={core.id} state={core} />
        ))}
      </div>
      <Bus />
    </div>
  );
};

export default App;

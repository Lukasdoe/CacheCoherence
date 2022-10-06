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

  const handleKeydown = (e: any) => {
    if (e.key === "ArrowRight") {
      next();
    }
  };

  const processLoad = (data: any) => {
    if (Object.hasOwn(data, "EnvInfo")) {
      const parsed_data = data.EnvInfo as EnvInfo;
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

  const processData = (data_list: [any]) => {
    let core_state = cores;
    for (const data of data_list) {
      switch (Object.keys(data)[0]) {
        case "Step":
          {
            let parsed_data = data.Step as Step;
            core_state = core_state.map((c, i) => {
              return {
                ...c,
                system: {
                  ...c.system,
                  clk: parsed_data.clk,
                },
              };
            });
            console.log(parsed_data.clk);
          }
          break;
        case "CoreState":
          {
            let parsed_data = data.CoreState as APICoreState;
            core_state = core_state.map((c, i) => {
              return i == parsed_data.id
                ? {
                    ...c,
                    record: parsed_data.record,
                    alu_cnt: parsed_data.alu_cnt,
                  }
                : c;
            });
          }
          break;
        case "CacheState":
          {
            let parsed_data = data.CacheState as CacheState;
            core_state = core_state.map((c, i) => {
              return i == parsed_data.core_id
                ? {
                    ...c,
                    cache: {
                      ...c.cache,
                      cnt: parsed_data.cnt,
                    },
                  }
                : c;
            });
          }
          break;
        case "CacheAccess":
          {
            let parsed_data = data.CacheAccess as CacheAccess;
            core_state = core_state.map((c, i) => {
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
            });
          }
          break;
        case "CacheUpdate":
          {
            let parsed_data = data.CacheUpdate as CacheUpdate;
            core_state = core_state.map((c, i) => {
              if (i == parsed_data.core_id) {
                c.cache.inv[parsed_data.index][parsed_data.block] =
                  parsed_data.new_tag;
              }
              return c;
            });
          }
          break;
        case "CoreInit":
          break;
      }
    }
    setCores(core_state);
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
    <div className="app" tabIndex={0} onKeyDown={handleKeydown}>
      <Header load={load} next={next} cycle={cores[0].system.clk} />
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

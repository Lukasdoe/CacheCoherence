import { useState, Component } from "react";

type CoreProps = {
  state: CoreState;
};

type CacheState = {
  cnt: number;
  hit: boolean;
  miss: boolean;
  tag: number;
  index: number;
  inv: number[][];
};

type SystemState = {
  protocol: string;
  cache_size: number;
  associativity: number;
  block_size: number;
  num_cores: number;
  clk: number;
};

export type CoreState = {
  system: SystemState;
  id: number;
  record?: string;
  alu_cnt: number;
  cache: CacheState;
};

export const loadSets = (
  cacheSize: number,
  associativity: number,
  blockSize: number
) => {
  const setSize = associativity * blockSize;
  const numberSets = cacheSize / setSize;

  const sets: number[][] = [];
  for (let i = 0; i < numberSets; i++) {
    const blocks: number[] = [];
    for (let i = 0; i < associativity; i++) {
      blocks.push(0);
    }
    sets.push(blocks);
  }
  return sets;
};

class Core extends Component<CoreProps> {
  render() {
    return (
      <div className="core">
        <div className="core-container">
          <div className="core-info">
            Core: {this.props.state.id}
            <table>
              <tbody>
                <tr>
                  <th>Alu</th>
                  <td>
                    {this.props.state.alu_cnt + this.props.state.cache.cnt}
                  </td>
                  <th colSpan={3} style={{ textAlign: "right" }}>
                    Address
                  </th>
                </tr>
                <tr>
                  <th>Type</th>
                  <td>{this.props.state.record?.split(" ")[0]}</td>
                  <td colSpan={3} style={{ textAlign: "right" }}>
                    {this.props.state.record?.split(" ")[1]}
                  </td>
                </tr>
                <tr></tr>
                <tr>
                  <th>Hit</th>
                  <td>{this.props.state.cache.hit ? "x" : ""}</td>
                  <th style={{ textAlign: "right" }}>Tag</th>
                  <th style={{ textAlign: "right" }}>Index</th>
                  <th style={{ textAlign: "right" }}>Block</th>
                </tr>
                <tr>
                  <th>Miss</th>
                  <td>{this.props.state.cache.miss ? "x" : ""}</td>
                  <th style={{ textAlign: "right" }}>
                    {this.props.state.cache.tag}
                  </th>
                  <th style={{ textAlign: "right" }}>
                    {this.props.state.cache.index}
                  </th>
                  <th style={{ textAlign: "right" }}>{"no block data"}</th>
                </tr>
              </tbody>
            </table>
          </div>
          <div className="core-cache">
            Cache
            <div className="core-cache-container">
              <table>
                <thead>
                  <tr>
                    <th>Set</th>
                    <th>Block</th>
                    <th>Tag</th>
                  </tr>
                </thead>
                <tbody>
                  {this.props.state.cache.inv.map((set, i) =>
                    set.map((block, j) => (
                      <tr key={i + j}>
                        <td>{j == 0 ? i : ""}</td>
                        <td>{j}</td>
                        <td>{block}</td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>
        <div className="core-bus-connector"></div>
      </div>
    );
  }
}

export default Core;

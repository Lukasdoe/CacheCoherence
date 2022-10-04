import { useState } from "react";
import CoreState from "../interfaces/core";
import System from "../interfaces/system";

type Props = {
  state: CoreState;
  system: System;
};

const loadSets = (
  cacheSize: number,
  associativity: number,
  blockSize: number
) => {
  const setSize = associativity * blockSize;
  const numberSets = cacheSize / setSize;

  const sets: number[][][] = [];
  for (let i = 0; i < numberSets; i++) {
    const blocks: number[][] = [];
    for (let i = 0; i < associativity; i++) {
      const words: number[] = [];
      for (let k = 0; k < Math.floor(blockSize / 4); k++) {
        words.push(0);
      }
      blocks.push(words);
    }
    sets.push(blocks);
  }

  return sets;
};

const Core = ({ state, system }: Props) => {
  const [cacheSets, setCacheSets] = useState<number[][][]>(
    loadSets(system.cache_size, system.associativity, system.block_size)
  );

  return (
    <div className="core">
      <div className="core-container">
        <div className="core-info">
          Core: {state.id}
          <table>
            <tbody>
              <tr>
                <th>Alu</th>
                <td>{state.alu + state.cache.alu}</td>
                <th colSpan={3} style={{ textAlign: "right" }}>
                  Address
                </th>
              </tr>
              <tr>
                <th>Type</th>
                <td>{state.record?.label}</td>
                <td colSpan={3} style={{ textAlign: "right" }}>
                  {state.record?.value.toString(16)}
                </td>
              </tr>
              <tr></tr>
              <tr>
                <th>Hit</th>
                <td>{state.cache.hit}</td>
                <th style={{ textAlign: "right" }}>Tag</th>
                <th style={{ textAlign: "right" }}>Index</th>
                <th style={{ textAlign: "right" }}>Block</th>
              </tr>
              <tr>
                <th>Miss</th>
                <td>{state.cache.miss}</td>
                <th style={{ textAlign: "right" }}>{state.cache.tag}</th>
                <th style={{ textAlign: "right" }}>{state.cache.index}</th>
                <th style={{ textAlign: "right" }}>
                  {state.cache.block_offset}
                </th>
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
                  <th>Word</th>
                </tr>
              </thead>
              <tbody>
                {cacheSets.map((set, i) =>
                  set.map((block, j) =>
                    block.map((word, k) => (
                      <tr key={i + j + k}>
                        <td>{j + k == 0 ? i : ""}</td>
                        <td>{k == 0 ? j : ""}</td>
                        <td>{word}</td>
                      </tr>
                    ))
                  )
                )}
              </tbody>
            </table>
          </div>
        </div>
      </div>
      <div className="core-bus-connector"></div>
    </div>
  );
};

export default Core;

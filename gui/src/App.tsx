import React, { useEffect, useState } from "react";
import Header from "./components/header";
import Core from "./components/core";
import Bus from "./components/bus";
import CoreState from "./interfaces/core";
import System from "./interfaces/system";

const defaultSystem = {
  protocol: "Mesi",
  cache_size: 256,
  associativity: 2,
  block_size: 8,
};

const App = () => {
  const [cores, setCores] = useState<CoreState[]>([]);
  const [system, setSystem] = useState<System>(defaultSystem);

  const load = () => {
    const postData = {
      protocol: "Mesi",
      cache_size: 256,
      associativity: 2,
      block_size: 8,
    };
    fetch("http://127.0.0.1:8080/load", {
      method: "POST",
      body: JSON.stringify({
        input_file: "data/02",
        ...postData,
      }),
    })
      .then((response) => response.json())
      .then((data) => {
        setSystem(postData);
        setCores(data);
      });
  };

  const step = () => {
    fetch("http://127.0.0.1:8080/step")
      .then((response) => response.json())
      .then((data) => setCores(data));
  };

  return (
    <div className="app">
      <Header load={load} step={step} />
      <div className="cores">
        {cores.map((core) => (
          <Core key={core.id} state={core} system={system} />
        ))}
      </div>
      <Bus />
    </div>
  );
};

export default App;

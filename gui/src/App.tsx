import React, { useEffect, useState } from "react";

const App = () => {
  const [cores, setCores] = useState<number>(0);

  useEffect(() => {
    fetch("http://127.0.0.1:8080/cores")
      .then((response) => response.json())
      .then((data) => setCores(data.cores));
  }, []);

  return <div>{cores}</div>;
};

export default App;

import CoreState from "../interfaces/core";
import System from "../interfaces/system";

const items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1, 1, 1, 1, 1, 1, 1];

type Props = {
  state: CoreState;
  system: System;
};

const Core = ({ state, system }: Props) => {
  return (
    <div className="core">
      <div className="core-container">
        <div className="core-info">
          Core: {state.id}
          <table>
            <tbody>
              <tr>
                <th>Alu</th>
                <td>{state.cnt}</td>
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
                <th></th>
                <td></td>
                <th style={{ textAlign: "right" }}>Tag</th>
                <th style={{ textAlign: "right" }}>Index</th>
                <th style={{ textAlign: "right" }}>Block</th>
              </tr>
              <tr>
                <th></th>
                <td></td>
                <th style={{ textAlign: "right" }}>0</th>
                <th style={{ textAlign: "right" }}>0</th>
                <th style={{ textAlign: "right" }}>0</th>
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
                  <th>Set </th>
                  <th>Set </th>
                  <th>Set </th>
                </tr>
              </thead>
              <tbody>
                {items.map((item) => (
                  <tr>
                    <td>0</td>
                    <td>1</td>
                    <td>2</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
      <div className="core-bus-connector">test</div>
    </div>
  );
};

export default Core;

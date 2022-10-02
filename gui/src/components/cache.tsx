type Props = {};

const items = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

const Cache = ({}: Props) => {
  return (
    <table className="core-cache-table">
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
  );
};

export default Cache;

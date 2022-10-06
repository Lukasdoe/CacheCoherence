type Props = {
  next: () => void;
  load: () => void;
  cycle: number;
};

const Header = ({ next, load, cycle }: Props) => {
  return (
    <header>
      <button onClick={load}>load</button>
      <button onClick={next}>next</button>
      <span>Current Cycle: {cycle}</span>
    </header>
  );
};

export default Header;

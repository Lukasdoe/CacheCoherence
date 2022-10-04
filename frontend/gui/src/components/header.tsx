type Props = {
  load: () => void;
  step: () => void;
};

const Header = ({ load, step }: Props) => {
  return (
    <header>
      <button onClick={load}>load</button>
      <button onClick={step}>step</button>
    </header>
  );
};

export default Header;

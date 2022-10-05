type Props = {
  next: () => void;
  load: () => void;
};

const Header = ({ next, load }: Props) => {
  return (
    <header>
      <button onClick={load}>load</button>
      <button onClick={next}>next</button>
    </header>
  );
};

export default Header;

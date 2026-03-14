import { useRef, useEffect } from "preact/hooks";

interface Props {
  value: string;
  onInput: (value: string) => void;
  onKeyDown: (e: KeyboardEvent) => void;
}

export function SearchInput({ value, onInput, onKeyDown }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div class="search-input-wrapper">
      <span class="search-icon">&#x1F50D;</span>
      <input
        ref={inputRef}
        type="text"
        class="omni-input"
        placeholder="Search files, apps, or type a command..."
        value={value}
        onInput={(e) => onInput((e.target as HTMLInputElement).value)}
        onKeyDown={onKeyDown}
      />
    </div>
  );
}

import { useRef, useEffect } from "preact/hooks";
import { listen } from "@tauri-apps/api/event";

interface Props {
  value: string;
  onInput: (value: string) => void;
  onKeyDown: (e: KeyboardEvent) => void;
}

export function SearchInput({ value, onInput, onKeyDown }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Re-focus when window is shown via Alt+Space
  useEffect(() => {
    const unlisten = listen("window-shown", () => {
      setTimeout(() => inputRef.current?.focus(), 50);
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  return (
    <div class="search-input-wrapper">
      <img src="/omni-icon.svg" class="search-icon-img" />
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

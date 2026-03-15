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
      // Focus immediately, then again after a tick in case Tauri's focus isn't ready
      inputRef.current?.focus();
      requestAnimationFrame(() => inputRef.current?.focus());
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  return (
    <div class="search-input-wrapper">
      <img src="/omni-icon.png" class="search-icon-img" />
      <input
        ref={inputRef}
        type="text"
        class="omni-input"
        autoFocus
        placeholder="Search files, apps, or type a command..."
        value={value}
        onInput={(e) => onInput((e.target as HTMLInputElement).value)}
        onKeyDown={onKeyDown}
      />
    </div>
  );
}

import { useState, useCallback, useRef, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SearchInput } from "./components/SearchInput";
import { ResultGroup } from "./components/ResultGroup";

interface SearchResult {
  category: string;
  title: string;
  subtitle: string;
  action: any;
  icon: string;
}

const CATEGORY_ORDER = ["Math", "Apps", "System", "Files", "URL", "Web"];

export function App() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [copiedFlash, setCopiedFlash] = useState(false);
  const debounceRef = useRef<number | null>(null);

  const grouped = CATEGORY_ORDER.map((cat) => ({
    category: cat,
    results: results.filter((r) => r.category === cat),
  })).filter((g) => g.results.length > 0);

  const flatResults = grouped.flatMap((g) => g.results);

  const handleInput = useCallback((value: string) => {
    setQuery(value);
    setSelectedIndex(0);

    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    if (!value.trim()) {
      setResults([]);
      return;
    }

    debounceRef.current = window.setTimeout(async () => {
      try {
        const res = await invoke<SearchResult[]>("search", { query: value });
        setResults(res);
      } catch (e) {
        console.error("Search error:", e);
      }
    }, 50);
  }, []);

  const DESTRUCTIVE_COMMANDS = ["shutdown", "restart", "sign_out"];

  const executeResult = useCallback(
    async (index: number) => {
      const result = flatResults[index];
      if (!result) return;

      if (result.action.type === "system_command" &&
          DESTRUCTIVE_COMMANDS.includes(result.action.command)) {
        const confirmed = window.confirm(`Are you sure you want to ${result.title.toLowerCase()}?`);
        if (!confirmed) return;
      }

      if (result.action.type === "copy") {
        await navigator.clipboard.writeText(result.action.text);
        setCopiedFlash(true);
        setTimeout(() => setCopiedFlash(false), 1000);
        return;
      }

      try {
        await invoke("execute_action", { action: result.action });
      } catch (e) {
        console.error("Action error:", e);
      }

      // Hide window after action
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().hide();
      setQuery("");
      setResults([]);
    },
    [flatResults]
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((i) => Math.min(i + 1, flatResults.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((i) => Math.max(i - 1, 0));
          break;
        case "Enter":
          e.preventDefault();
          executeResult(selectedIndex);
          break;
        case "Tab":
          e.preventDefault();
          // Jump to next category
          let currentGroup = 0;
          let count = 0;
          for (const g of grouped) {
            if (count + g.results.length > selectedIndex) {
              currentGroup = grouped.indexOf(g);
              break;
            }
            count += g.results.length;
          }
          const nextGroup = (currentGroup + 1) % grouped.length;
          let nextIndex = 0;
          for (let i = 0; i < nextGroup; i++) {
            nextIndex += grouped[i].results.length;
          }
          setSelectedIndex(nextIndex);
          break;
        case "Escape":
          e.preventDefault();
          invoke("hide_window");
          break;
      }
    },
    [flatResults, selectedIndex, grouped, executeResult]
  );

  // Scroll selected item into view
  useEffect(() => {
    const el = document.querySelector(".result-item.selected");
    el?.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  // Resize window — anchor top position, only grow downward
  useEffect(() => {
    (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const { LogicalSize } = await import("@tauri-apps/api/dpi");
        const win = getCurrentWindow();

        let targetHeight: number;
        if (flatResults.length === 0 && !query.trim()) {
          targetHeight = 68;
        } else if (flatResults.length === 0) {
          targetHeight = 130;
        } else {
          const base = 56 + 16 + 2 + 16 + 16;
          const groupCost = grouped.length * 38;
          const resultCost = flatResults.length * 44;
          targetHeight = base + groupCost + resultCost;
          targetHeight = Math.min(targetHeight, 800);
        }

        // Only resize height, don't re-center — keeps search bar anchored
        await win.setSize(new LogicalSize(600, targetHeight));
      } catch (e) {
        console.error(`[Omni resize] ERROR:`, e);
      }
    })();
  }, [flatResults.length, grouped.length, query]);

  // Listen for backend events
  useEffect(() => {
    const unlistenClear = listen("clear-query", () => {
      setQuery("");
      setResults([]);
      setSelectedIndex(0);
    });
    const unlistenShown = listen("window-shown", () => {
      invoke("refresh_apps");
    });
    return () => {
      unlistenClear.then((fn) => fn());
      unlistenShown.then((fn) => fn());
    };
  }, []);

  let globalIndex = 0;

  return (
    <div class="omni-container">
      <SearchInput value={query} onInput={handleInput} onKeyDown={handleKeyDown} />
      {flatResults.length > 0 ? (
        <div class="results-container">
          {grouped.map((group) => {
            const startIndex = globalIndex;
            globalIndex += group.results.length;
            return (
              <ResultGroup
                key={group.category}
                category={group.category}
                results={group.results}
                selectedIndex={selectedIndex}
                globalStartIndex={startIndex}
                onExecute={executeResult}
              />
            );
          })}
          {copiedFlash && <div class="copied-flash">Copied!</div>}
        </div>
      ) : query.trim() ? (
        <div class="empty-state">No results found</div>
      ) : null}
    </div>
  );
}

import { useState, useCallback, useRef, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SearchInput } from "./components/SearchInput";
import { ResultGroup } from "./components/ResultGroup";
import { ContextMenu, getActions } from "./components/ContextMenu";

interface SearchResult {
  category: string;
  title: string;
  subtitle: string;
  action: any;
  icon: string;
}

const CATEGORY_ORDER = ["Math", "Apps", "System", "Files", "Directories", "URL", "Web"];

export function App() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [copiedFlash, setCopiedFlash] = useState(false);
  const [expandedCategory, setExpandedCategory] = useState<string | null>(null);
  const [contextMenuIndex, setContextMenuIndex] = useState<number | null>(null);
  const [contextActionIndex, setContextActionIndex] = useState(0);
  const [showHelp, setShowHelp] = useState(false);
  const debounceRef = useRef<number | null>(null);

  const grouped = CATEGORY_ORDER.map((cat) => ({
    category: cat,
    results: results.filter((r) => r.category === cat),
  })).filter((g) => g.results.length > 0);

  const flatResults = grouped.flatMap((g) => g.results);

  const handleInput = useCallback((value: string) => {
    setQuery(value);
    setSelectedIndex(0);
    setExpandedCategory(null);
    setContextMenuIndex(null);
    setContextActionIndex(0);

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

  // Find which category the selected index belongs to
  const getSelectedCategory = useCallback((): string | null => {
    let count = 0;
    for (const g of grouped) {
      if (count + g.results.length > selectedIndex) {
        return g.category;
      }
      count += g.results.length;
    }
    return null;
  }, [grouped, selectedIndex]);

  const expandCategory = useCallback(async () => {
    const cat = getSelectedCategory();
    if (!cat || cat === "Math" || cat === "URL") return; // these don't expand

    try {
      const expanded = await invoke<SearchResult[]>("expand_category", {
        query,
        category: cat,
      });
      if (expanded.length > 0) {
        // Replace results in this category with expanded results
        setResults((prev) => {
          const other = prev.filter((r) => r.category !== cat);
          return [...other, ...expanded];
        });
        setExpandedCategory(cat);
      }
    } catch (e) {
      console.error("Expand error:", e);
    }
  }, [query, getSelectedCategory]);

  const executeContextAction = useCallback(async (actionIndex: number) => {
    const result = flatResults[contextMenuIndex!];
    if (!result) return;
    const actions = getActions(result);
    if (actionIndex >= 0 && actionIndex < actions.length) {
      try {
        await actions[actionIndex].handler();
      } catch (e) {
        console.error("Context action error:", e);
      }
    }
    setContextMenuIndex(null);
    setContextActionIndex(0);
  }, [flatResults, contextMenuIndex]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      // Context menu is open — handle its navigation
      if (contextMenuIndex !== null) {
        const result = flatResults[contextMenuIndex];
        const actions = result ? getActions(result) : [];

        switch (e.key) {
          case "ArrowDown":
            e.preventDefault();
            setContextActionIndex((i) => Math.min(i + 1, actions.length - 1));
            return;
          case "ArrowUp":
            e.preventDefault();
            setContextActionIndex((i) => Math.max(i - 1, 0));
            return;
          case "Enter":
            e.preventDefault();
            executeContextAction(contextActionIndex);
            return;
          case "ArrowLeft":
            if (e.shiftKey) {
              e.preventDefault();
              setContextMenuIndex(null);
              setContextActionIndex(0);
            }
            return;
          case "Escape":
            e.preventDefault();
            setContextMenuIndex(null);
            setContextActionIndex(0);
            return;
          default:
            return;
        }
      }

      // Helper: find start/end index of current category group
      const getCategoryBounds = () => {
        let start = 0;
        for (const g of grouped) {
          const end = start + g.results.length - 1;
          if (selectedIndex >= start && selectedIndex <= end) {
            return { start, end };
          }
          start = end + 1;
        }
        return { start: 0, end: flatResults.length - 1 };
      };

      // Helper: get the start index of the next/prev category
      const getNextCategoryStart = () => {
        let start = 0;
        let foundCurrent = false;
        for (const g of grouped) {
          if (foundCurrent) return start;
          const end = start + g.results.length - 1;
          if (selectedIndex >= start && selectedIndex <= end) {
            foundCurrent = true;
          }
          start = end + 1;
        }
        return flatResults.length - 1; // stay at end if last category
      };

      const getPrevCategoryEnd = () => {
        let start = 0;
        let prevEnd = 0;
        for (const g of grouped) {
          const end = start + g.results.length - 1;
          if (selectedIndex >= start && selectedIndex <= end) {
            return prevEnd > 0 ? prevEnd : 0;
          }
          prevEnd = end;
          start = end + 1;
        }
        return 0;
      };

      // Normal result navigation
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          if (e.ctrlKey) {
            const bounds = getCategoryBounds();
            if (selectedIndex === bounds.end) {
              // Already at end of category — jump to next category
              setSelectedIndex(Math.min(getNextCategoryStart(), flatResults.length - 1));
            } else {
              setSelectedIndex(bounds.end);
            }
          } else {
            setSelectedIndex((i) => Math.min(i + 1, flatResults.length - 1));
          }
          break;
        case "ArrowUp":
          e.preventDefault();
          if (e.ctrlKey) {
            const bounds = getCategoryBounds();
            if (selectedIndex === bounds.start) {
              // Already at start of category — jump to prev category end
              setSelectedIndex(Math.max(getPrevCategoryEnd(), 0));
            } else {
              setSelectedIndex(bounds.start);
            }
          } else {
            setSelectedIndex((i) => Math.max(i - 1, 0));
          }
          break;
        case "Home":
          e.preventDefault();
          setSelectedIndex(0);
          break;
        case "End":
          e.preventDefault();
          setSelectedIndex(flatResults.length - 1);
          break;
        case "PageDown": {
          e.preventDefault();
          const bounds = getCategoryBounds();
          if (selectedIndex === bounds.end) {
            setSelectedIndex(Math.min(getNextCategoryStart(), flatResults.length - 1));
          } else {
            setSelectedIndex(bounds.end);
          }
          break;
        }
        case "PageUp": {
          e.preventDefault();
          const bounds = getCategoryBounds();
          if (selectedIndex === bounds.start) {
            setSelectedIndex(Math.max(getPrevCategoryEnd(), 0));
          } else {
            setSelectedIndex(bounds.start);
          }
          break;
        }
        case "ArrowRight":
          // Shift+Right: open context menu
          if (e.shiftKey && flatResults.length > 0) {
            const result = flatResults[selectedIndex];
            if (result && getActions(result).length > 0) {
              e.preventDefault();
              setContextMenuIndex(selectedIndex);
              setContextActionIndex(0);
            }
          }
          break;
        case "Enter":
          e.preventDefault();
          executeResult(selectedIndex);
          break;
        case "Tab":
          e.preventDefault();
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
        case "e":
        case "E":
          if (e.ctrlKey && flatResults.length > 0) {
            e.preventDefault();
            expandCategory();
          }
          break;
        case "h":
        case "H":
          if (e.ctrlKey) {
            e.preventDefault();
            setShowHelp((v) => !v);
          }
          break;
        case "Escape":
          e.preventDefault();
          if (showHelp) {
            setShowHelp(false);
          } else if (expandedCategory) {
            setExpandedCategory(null);
            invoke<SearchResult[]>("search", { query }).then(setResults);
          } else {
            invoke("hide_window");
          }
          break;
      }
    },
    [flatResults, selectedIndex, grouped, executeResult, expandCategory, expandedCategory, query, contextMenuIndex, contextActionIndex, executeContextAction, showHelp]
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
          targetHeight = 52;
        } else if (flatResults.length === 0) {
          targetHeight = 110;
        } else {
          const base = 52 + 24;
          const groupCost = grouped.length * 38;
          const resultCost = flatResults.length * 44;
          targetHeight = base + groupCost + resultCost;
          // Allow up to 80% of screen height
          const maxH = window.screen.availHeight * 0.75;
          targetHeight = Math.min(targetHeight, maxH);
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
  const activeCategory = getSelectedCategory();

  return (
    <div class="omni-container">
      <SearchInput value={query} onInput={handleInput} onKeyDown={handleKeyDown} />
      {contextMenuIndex !== null && flatResults[contextMenuIndex] ? (
        <div class="results-container">
          <ContextMenu
            result={flatResults[contextMenuIndex]}
            selectedAction={contextActionIndex}
            onExecute={executeContextAction}
          />
        </div>
      ) : flatResults.length > 0 ? (
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
                isActive={group.category === activeCategory}
                isExpanded={group.category === expandedCategory}
              />
            );
          })}
          {copiedFlash && <div class="copied-flash">Copied!</div>}
        </div>
      ) : query.trim() ? (
        <div class="empty-state">No results found</div>
      ) : null}
      {showHelp && (
        <div class="help-overlay">
          <div class="help-title">Keyboard Shortcuts <span class="help-dismiss">Ctrl+H to close</span></div>
          <div class="help-grid">
            <div class="help-section">
              <div class="help-section-title">Navigation</div>
              <div class="help-row"><kbd>↑ ↓</kbd><span>Move between results</span></div>
              <div class="help-row"><kbd>Tab</kbd><span>Jump to next category</span></div>
              <div class="help-row"><kbd>Ctrl+↑</kbd><span>Category start, then prev</span></div>
              <div class="help-row"><kbd>Ctrl+↓</kbd><span>Category end, then next</span></div>
              <div class="help-row"><kbd>Home</kbd><span>First result</span></div>
              <div class="help-row"><kbd>End</kbd><span>Last result</span></div>
              <div class="help-row"><kbd>PgUp/PgDn</kbd><span>Category bounds</span></div>
            </div>
            <div class="help-section">
              <div class="help-section-title">Actions</div>
              <div class="help-row"><kbd>Enter</kbd><span>Open / execute</span></div>
              <div class="help-row"><kbd>Shift+→</kbd><span>Context menu</span></div>
              <div class="help-row"><kbd>Shift+←</kbd><span>Close context menu</span></div>
              <div class="help-row"><kbd>Ctrl+E</kbd><span>Expand category (50 results)</span></div>
              <div class="help-row"><kbd>Escape</kbd><span>Collapse / hide</span></div>
              <div class="help-row"><kbd>Ctrl+H</kbd><span>Toggle this help</span></div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

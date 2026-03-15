import { useState, useCallback, useRef, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SearchInput } from "./components/SearchInput";
import { ResultGroup } from "./components/ResultGroup";
import { ContextMenu, getActions } from "./components/ContextMenu";
import { PreviewPanel } from "./components/PreviewPanel";
import type { FilePreview } from "./components/PreviewPanel";
import { TablePanel } from "./components/TablePanel";
import type { TableResult, SortColumn } from "./components/TablePanel";

interface SearchResult {
  category: string;
  title: string;
  subtitle: string;
  action: any;
  icon: string;
  size?: number;
  date_modified?: number;
}

function BatchContextMenu({ count, selectedAction, onExecute }: { count: number; selectedAction: number; onExecute: (i: number) => void }) {
  const labels = [
    `Open all (${count} items)`,
    "Copy all paths",
    "Copy all to...",
    "Move all to...",
    "Delete all (recycle bin)",
  ];
  return (
    <div class="context-menu">
      <div class="context-menu-header">
        Batch actions ({count} items)
        <span class="context-hint">Shift+\u2190 back</span>
      </div>
      {labels.map((label, i) => (
        <div
          key={i}
          class={`context-action ${i === selectedAction ? "selected" : ""}`}
          onClick={() => onExecute(i)}
        >
          {label}
        </div>
      ))}
    </div>
  );
}

const CATEGORY_ORDER = ["Frequent", "Math", "Color", "Apps", "System", "Clipboard", "Processes", "Files", "Directories", "URL", "Web"];

export function App() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [copiedFlash, setCopiedFlash] = useState(false);
  const [expandedCategory, setExpandedCategory] = useState<string | null>(null);
  const [contextMenuIndex, setContextMenuIndex] = useState<number | null>(null);
  const [contextActionIndex, setContextActionIndex] = useState(0);
  const [showHelp, setShowHelp] = useState(false);
  const [previewData, setPreviewData] = useState<FilePreview | null>(null);
  const [completionCandidates, setCompletionCandidates] = useState<string[]>([]);
  const [completionIndex, setCompletionIndex] = useState(0);
  const [multiSelected, setMultiSelected] = useState<Set<number>>(new Set());
  const [tableOpen, setTableOpen] = useState(false);
  const [activePanel, setActivePanel] = useState<"results" | "table">("results");
  const [tableSelectedIndex, setTableSelectedIndex] = useState(0);
  const [tableResults, setTableResults] = useState<TableResult[]>([]);
  const [tableMultiSelected, setTableMultiSelected] = useState<Set<number>>(new Set());
  const [tableSortColumn, setTableSortColumn] = useState<SortColumn>("date_modified");
  const [tableSortAscending, setTableSortAscending] = useState(false);
  const [tableContextResult, setTableContextResult] = useState<SearchResult | null>(null);
  const [originalWindowPos, setOriginalWindowPos] = useState<{ x: number; y: number } | null>(null);
  const debounceRef = useRef<number | null>(null);

  const grouped = CATEGORY_ORDER.map((cat) => ({
    category: cat,
    results: results.filter((r) => r.category === cat),
  })).filter((g) => g.results.length > 0);

  // When table is open, filter Files/Directories from the left panel (they live in the table)
  const leftGrouped = tableOpen
    ? grouped.filter(g => g.category !== "Files" && g.category !== "Directories")
    : grouped;

  const flatResults = leftGrouped.flatMap((g) => g.results);

  const fetchTableResults = useCallback(async (q: string, sortBy = "date_modified", asc = false) => {
    if (!q.trim()) return;
    try {
      const res = await invoke<TableResult[]>("search_table", { query: q, sortBy, ascending: asc });
      setTableResults(res);
      setTableSelectedIndex(0);
      setTableMultiSelected(new Set());
    } catch (e) {
      console.error("Table search error:", e);
    }
  }, []);

  const toggleTable = useCallback(async () => {
    if (tableOpen) {
      // Close table — restore position
      setTableOpen(false);
      setActivePanel("results");
      setTableMultiSelected(new Set());
      if (originalWindowPos) {
        try {
          const { getCurrentWindow } = await import("@tauri-apps/api/window");
          const { LogicalPosition } = await import("@tauri-apps/api/dpi");
          const win = getCurrentWindow();
          await win.setPosition(new LogicalPosition(originalWindowPos.x, originalWindowPos.y));
          setOriginalWindowPos(null);
        } catch (e) {
          console.error("Table close error:", e);
        }
      }
    } else {
      // Open table — resize effect handles the window sizing
      const hasFileResults = results.some(r => r.category === "Files" || r.category === "Directories");
      if (!hasFileResults) return;
      setTableOpen(true);
      setActivePanel("table");
      fetchTableResults(query);
    }
  }, [tableOpen, results, query, originalWindowPos, fetchTableResults]);

  const handleInput = useCallback((value: string) => {
    setQuery(value);
    setSelectedIndex(0);
    setExpandedCategory(null);
    setContextMenuIndex(null);
    setContextActionIndex(0);
    setPreviewData(null);
    setCompletionCandidates([]);
    setCompletionIndex(0);
    setMultiSelected(new Set());

    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    if (!value.trim()) {
      if (tableOpen) {
        toggleTable();
      }
      // Show frequent items when query is cleared
      invoke<SearchResult[]>("get_frequent_items")
        .then((frequent) => setResults(frequent.length > 0 ? frequent : []))
        .catch(() => setResults([]));
      return;
    }

    debounceRef.current = window.setTimeout(async () => {
      try {
        const res = await invoke<SearchResult[]>("search", { query: value });
        setResults(res);
        // Auto-open table if results contain files/directories
        const hasFiles = res.some(r => r.category === "Files" || r.category === "Directories");
        if (hasFiles && !tableOpen) {
          // Open table without the full toggleTable resize dance — just fetch and show
          setTableOpen(true);
          fetchTableResults(value);
        } else if (hasFiles && tableOpen) {
          fetchTableResults(value);
        } else if (!hasFiles && tableOpen) {
          setTableOpen(false);
          setActivePanel("results");
        }
      } catch (e) {
        console.error("Search error:", e);
      }
    }, 50);
  }, [tableOpen, fetchTableResults, toggleTable]);

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

      if (result.action.type === "kill_process") {
        const confirmed = window.confirm(`Kill process "${result.action.name}" (PID ${result.action.pid})?`);
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

      // Record usage for ranking boost
      invoke("record_selection", {
        query,
        resultPath: result.subtitle,
        category: result.category,
        title: result.title,
      });

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
    for (const g of leftGrouped) {
      if (count + g.results.length > selectedIndex) {
        return g.category;
      }
      count += g.results.length;
    }
    return null;
  }, [leftGrouped, selectedIndex]);

  const expandCategory = useCallback(async () => {
    const cat = getSelectedCategory();
    if (!cat || cat === "Math" || cat === "URL") return;

    // Toggle: if already expanded, collapse back
    if (expandedCategory === cat) {
      setExpandedCategory(null);
      try {
        const res = await invoke<SearchResult[]>("search", { query });
        setResults(res);
      } catch (e) {
        console.error("Collapse error:", e);
      }
      return;
    }

    try {
      const expanded = await invoke<SearchResult[]>("expand_category", {
        query,
        category: cat,
      });
      if (expanded.length > 0) {
        setResults((prev) => {
          const other = prev.filter((r) => r.category !== cat);
          return [...other, ...expanded];
        });
        setExpandedCategory(cat);
      }
    } catch (e) {
      console.error("Expand error:", e);
    }
  }, [query, getSelectedCategory, expandedCategory]);

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
      // Preview is open — navigate within preview
      if (previewData) {
        const previewContent = document.querySelector(".preview-content");
        const scrollAmount = 100;
        switch (e.key) {
          case "Escape":
            e.preventDefault();
            setPreviewData(null);
            return;
          case "ArrowDown":
            e.preventDefault();
            if (e.ctrlKey) {
              previewContent?.scrollTo({ top: previewContent.scrollHeight, behavior: "smooth" });
            } else {
              previewContent?.scrollBy({ top: scrollAmount, behavior: "smooth" });
            }
            return;
          case "ArrowUp":
            e.preventDefault();
            if (e.ctrlKey) {
              previewContent?.scrollTo({ top: 0, behavior: "smooth" });
            } else {
              previewContent?.scrollBy({ top: -scrollAmount, behavior: "smooth" });
            }
            return;
          case "Home":
            e.preventDefault();
            previewContent?.scrollTo({ top: 0, behavior: "smooth" });
            return;
          case "End":
            e.preventDefault();
            previewContent?.scrollTo({ top: previewContent.scrollHeight, behavior: "smooth" });
            return;
          case "PageDown":
            e.preventDefault();
            previewContent?.scrollBy({ top: previewContent.clientHeight * 0.8, behavior: "smooth" });
            return;
          case "PageUp":
            e.preventDefault();
            previewContent?.scrollBy({ top: -(previewContent.clientHeight * 0.8), behavior: "smooth" });
            return;
          case " ":
            if (e.ctrlKey) {
              e.preventDefault();
              setPreviewData(null);
            }
            return;
          default:
            return;
        }
      }

      // Context menu is open — handle its navigation
      if (contextMenuIndex !== null) {
        const isBatch = contextMenuIndex === -1;
        const isTableContext = contextMenuIndex === -2;
        const result = isTableContext ? tableContextResult : (isBatch ? null : flatResults[contextMenuIndex]);
        const actionCount = isBatch ? 5 : (result ? getActions(result).length : 0);

        switch (e.key) {
          case "ArrowDown":
            e.preventDefault();
            setContextActionIndex((i) => Math.min(i + 1, actionCount - 1));
            return;
          case "ArrowUp":
            e.preventDefault();
            setContextActionIndex((i) => Math.max(i - 1, 0));
            return;
          case "Enter":
            e.preventDefault();
            if (isBatch) {
              // Batch context menu — trigger via the onExecute passed to BatchContextMenu
              // We need to simulate clicking the action; dispatch it here directly
              const indices = [...new Set([...multiSelected, selectedIndex])];
              const paths = indices.map(i => flatResults[i]?.subtitle).filter(Boolean);
              const batchHandlers = [
                () => invoke("batch_open", { paths }),
                () => navigator.clipboard.writeText(paths.join("\n")),
                () => invoke("batch_copy_to", { paths }),
                () => invoke("batch_move_to", { paths }),
                async () => {
                  const confirmed = window.confirm(`Move ${paths.length} items to recycle bin?`);
                  if (confirmed) await invoke("batch_delete", { paths });
                },
              ];
              if (contextActionIndex >= 0 && contextActionIndex < batchHandlers.length) {
                batchHandlers[contextActionIndex]();
              }
              setContextMenuIndex(null);
              setContextActionIndex(0);
              setMultiSelected(new Set());
            } else if (isTableContext && tableContextResult) {
              const actions = getActions(tableContextResult);
              if (contextActionIndex >= 0 && contextActionIndex < actions.length) {
                actions[contextActionIndex].handler();
              }
              setContextMenuIndex(null);
              setContextActionIndex(0);
              setTableContextResult(null);
            } else {
              executeContextAction(contextActionIndex);
            }
            return;
          case "ArrowLeft":
            if (e.shiftKey) {
              e.preventDefault();
              setContextMenuIndex(null);
              setContextActionIndex(0);
              setTableContextResult(null);
            }
            return;
          case "Escape":
            e.preventDefault();
            setContextMenuIndex(null);
            setContextActionIndex(0);
            setTableContextResult(null);
            return;
          default:
            return;
        }
      }

      // Table panel is focused — handle its navigation
      if (tableOpen && activePanel === "table") {
        switch (e.key) {
          case "ArrowDown":
            e.preventDefault();
            if (e.shiftKey) {
              setTableMultiSelected(prev => new Set([...prev, tableSelectedIndex]));
            }
            if (e.ctrlKey) {
              // Jump to last visible row (page-down style)
              const body = document.querySelector(".table-body") as HTMLElement | null;
              if (body) {
                const visibleRows = Math.floor(body.clientHeight / 27); // ~27px per row
                setTableSelectedIndex(i => Math.min(i + visibleRows, tableResults.length - 1));
              } else {
                setTableSelectedIndex(tableResults.length - 1);
              }
            } else {
              setTableSelectedIndex(i => Math.min(i + 1, tableResults.length - 1));
            }
            return;
          case "ArrowUp":
            e.preventDefault();
            if (e.shiftKey) {
              setTableMultiSelected(prev => new Set([...prev, tableSelectedIndex]));
            }
            if (e.ctrlKey) {
              // Jump up a page
              const body = document.querySelector(".table-body") as HTMLElement | null;
              if (body) {
                const visibleRows = Math.floor(body.clientHeight / 27);
                setTableSelectedIndex(i => Math.max(i - visibleRows, 0));
              } else {
                setTableSelectedIndex(0);
              }
            } else {
              setTableSelectedIndex(i => Math.max(i - 1, 0));
            }
            return;
          case "PageDown":
            e.preventDefault();
            {
              const body = document.querySelector(".table-body") as HTMLElement | null;
              const visibleRows = body ? Math.floor(body.clientHeight / 27) : 20;
              setTableSelectedIndex(i => Math.min(i + visibleRows, tableResults.length - 1));
            }
            return;
          case "PageUp":
            e.preventDefault();
            {
              const body = document.querySelector(".table-body") as HTMLElement | null;
              const visibleRows = body ? Math.floor(body.clientHeight / 27) : 20;
              setTableSelectedIndex(i => Math.max(i - visibleRows, 0));
            }
            return;
          case "Home":
            e.preventDefault();
            setTableSelectedIndex(0);
            return;
          case "End":
            e.preventDefault();
            setTableSelectedIndex(tableResults.length - 1);
            return;
          case "Enter":
            e.preventDefault();
            if (tableMultiSelected.size > 0) {
              const indices = new Set([...tableMultiSelected, tableSelectedIndex]);
              const paths = [...indices].map(i => tableResults[i]?.subtitle).filter(Boolean);
              invoke("batch_open", { paths });
              setTableMultiSelected(new Set());
            } else {
              const tr = tableResults[tableSelectedIndex];
              if (tr) {
                invoke("execute_action", { action: tr.action });
                invoke("record_selection", { query, resultPath: tr.subtitle, category: tr.category, title: tr.title });
              }
            }
            return;
          case "ArrowRight":
            if (e.shiftKey && tableResults.length > 0) {
              e.preventDefault();
              const tr = tableResults[tableSelectedIndex];
              if (tr) {
                setTableContextResult(tr as SearchResult);
                setContextMenuIndex(-2); // -2 signals table context menu
                setContextActionIndex(0);
              }
            }
            return;
          case "ArrowLeft":
            if (e.ctrlKey) {
              // Ctrl+Left: jump focus back to result list
              e.preventDefault();
              setActivePanel("results");
              setTableMultiSelected(new Set());
            }
            return;
          case " ":
            if (e.ctrlKey) {
              e.preventDefault();
              if (previewData) {
                setPreviewData(null);
              } else {
                const tr = tableResults[tableSelectedIndex];
                if (tr) {
                  invoke<FilePreview>("preview_file", { path: tr.subtitle })
                    .then(preview => setPreviewData(preview))
                    .catch(err => console.error("Preview error:", err));
                }
              }
            }
            return;
          case "Escape":
            e.preventDefault();
            if (showHelp) {
              setShowHelp(false);
            } else if (tableMultiSelected.size > 0) {
              setTableMultiSelected(new Set());
            } else {
              toggleTable();
            }
            return;
          case "h": case "H":
            if (e.ctrlKey) {
              e.preventDefault();
              setShowHelp(v => !v);
            }
            return;
          case "1": case "2": case "3": case "4":
            if (e.ctrlKey) {
              e.preventDefault();
              const cols: Array<SortColumn> = ["name", "path", "size", "date_modified"];
              const col = cols[parseInt(e.key) - 1];
              const asc = col === "name" || col === "path";
              setTableSortColumn(col);
              setTableSortAscending(asc);
              fetchTableResults(query, col, asc);
            }
            return;
          case "t": case "T":
            if (e.ctrlKey) {
              e.preventDefault();
              toggleTable();
            }
            return;
          case "Tab":
            if (e.ctrlKey) {
              e.preventDefault();
              setActivePanel("results");
              setTableMultiSelected(new Set());
            }
            return;
          case "f": case "F":
            if (e.ctrlKey) e.preventDefault(); // suppress Chromium "Find in page"
            return;
          default:
            return;
        }
      }

      // Helper: find start/end index of current category group
      const getCategoryBounds = () => {
        let start = 0;
        for (const g of leftGrouped) {
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
        for (const g of leftGrouped) {
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
        for (const g of leftGrouped) {
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
          if (e.shiftKey && !e.ctrlKey) {
            setMultiSelected(prev => new Set([...prev, selectedIndex]));
          }
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
          if (e.shiftKey && !e.ctrlKey) {
            setMultiSelected(prev => new Set([...prev, selectedIndex]));
          }
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
          // Shift+Right: open context menu (batch or single)
          if (e.shiftKey && flatResults.length > 0) {
            if (multiSelected.size > 0) {
              // Open batch context menu
              e.preventDefault();
              setContextMenuIndex(-1); // -1 signals batch mode
              setContextActionIndex(0);
            } else {
              const result = flatResults[selectedIndex];
              if (result && getActions(result).length > 0) {
                e.preventDefault();
                setContextMenuIndex(selectedIndex);
                setContextActionIndex(0);
              }
            }
          } else if (e.ctrlKey && tableOpen) {
            // Ctrl+Right: jump focus to table panel
            e.preventDefault();
            setActivePanel("table");
            setMultiSelected(new Set());
          }
          break;
        case "Enter":
          e.preventDefault();
          if (multiSelected.size > 0) {
            // Batch open: all multi-selected + current
            const indices = new Set([...multiSelected, selectedIndex]);
            const paths = [...indices].map(i => flatResults[i]?.subtitle).filter(Boolean);
            invoke("batch_open", { paths });
            setMultiSelected(new Set());
          } else {
            executeResult(selectedIndex);
          }
          break;
        case "Tab":
          if (e.ctrlKey && tableOpen) {
            e.preventDefault();
            setActivePanel(p => p === "results" ? "table" : "results");
            if (activePanel === "results") {
              setMultiSelected(new Set());
            } else {
              setTableMultiSelected(new Set());
            }
            break;
          }
          e.preventDefault();
          // Path completion: if query looks like a path
          if (/^[a-zA-Z]:[\\\/]/.test(query) || query.startsWith("\\\\")) {
            if (completionCandidates.length > 0) {
              // Cycle through existing candidates
              const nextIdx = e.shiftKey
                ? (completionIndex - 1 + completionCandidates.length) % completionCandidates.length
                : (completionIndex + 1) % completionCandidates.length;
              setCompletionIndex(nextIdx);
              setQuery(completionCandidates[nextIdx] + "\\");
            } else {
              // Fetch candidates
              invoke<string[]>("complete_path", { partial: query }).then((candidates) => {
                if (candidates.length > 0) {
                  setCompletionCandidates(candidates);
                  setCompletionIndex(0);
                  setQuery(candidates[0] + "\\");
                }
              });
            }
          } else {
            // Normal category jump
            let currentGroup = 0;
            let count = 0;
            for (const g of leftGrouped) {
              if (count + g.results.length > selectedIndex) {
                currentGroup = leftGrouped.indexOf(g);
                break;
              }
              count += g.results.length;
            }
            const nextGroup = (currentGroup + 1) % leftGrouped.length;
            let nextIndex = 0;
            for (let i = 0; i < nextGroup; i++) {
              nextIndex += leftGrouped[i].results.length;
            }
            setSelectedIndex(nextIndex);
          }
          break;
        case "c":
        case "C":
          if (e.ctrlKey && flatResults.length > 0) {
            e.preventDefault();
            const r = flatResults[selectedIndex];
            if (r) {
              navigator.clipboard.writeText(r.subtitle);
              setCopiedFlash(true);
              setTimeout(() => setCopiedFlash(false), 1000);
            }
          }
          break;
        case "e":
        case "E":
          if (e.ctrlKey && flatResults.length > 0) {
            e.preventDefault();
            if (tableOpen) {
              const cat = getSelectedCategory();
              if (cat === "Files" || cat === "Directories") break;
            }
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
        case "f":
        case "F":
          if (e.ctrlKey) {
            e.preventDefault(); // suppress Chromium "Find in page"
          }
          break;
        case " ":
          if (e.ctrlKey && flatResults.length > 0) {
            e.preventDefault();
            if (previewData) {
              setPreviewData(null);
            } else {
              const result = flatResults[selectedIndex];
              if (result && (result.category === "Files" || result.category === "Directories")) {
                invoke<FilePreview>("preview_file", { path: result.subtitle })
                  .then((preview) => setPreviewData(preview))
                  .catch((err) => console.error("Preview error:", err));
              }
            }
          }
          break;
        case "t":
        case "T":
          if (e.ctrlKey) {
            e.preventDefault();
            toggleTable();
          }
          break;
        case "Escape":
          e.preventDefault();
          if (multiSelected.size > 0) {
            setMultiSelected(new Set());
          } else if (showHelp) {
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
    [flatResults, selectedIndex, leftGrouped, executeResult, expandCategory, expandedCategory, query, contextMenuIndex, contextActionIndex, executeContextAction, showHelp, previewData, completionCandidates, completionIndex, multiSelected, tableOpen, activePanel, tableSelectedIndex, tableResults, tableMultiSelected, toggleTable, fetchTableResults]
  );

  // Scroll selected item into view with padding for group headers
  useEffect(() => {
    const el = document.querySelector(".result-item.selected") as HTMLElement | null;
    const container = document.querySelector(".results-container") as HTMLElement | null;
    if (!el || !container) return;

    const prev = el.previousElementSibling as HTMLElement | null;
    const target = (prev && prev.classList.contains("result-group-header")) ? prev : el;

    const containerRect = container.getBoundingClientRect();
    const targetRect = target.getBoundingClientRect();

    // If target is above the visible area, scroll up with 12px padding
    if (targetRect.top < containerRect.top) {
      container.scrollTop -= (containerRect.top - targetRect.top + 12);
    }
    // If selected item is below the visible area, scroll down
    else {
      const elRect = el.getBoundingClientRect();
      if (elRect.bottom > containerRect.bottom) {
        container.scrollTop += (elRect.bottom - containerRect.bottom + 8);
      }
    }
  }, [selectedIndex]);

  // Scroll selected table row into view
  useEffect(() => {
    if (!tableOpen) return;
    const el = document.querySelector(".table-row.selected") as HTMLElement | null;
    const container = document.querySelector(".table-body") as HTMLElement | null;
    if (!el || !container) return;

    const containerRect = container.getBoundingClientRect();
    const elRect = el.getBoundingClientRect();
    if (elRect.top < containerRect.top) {
      container.scrollTop -= (containerRect.top - elRect.top + 4);
    } else if (elRect.bottom > containerRect.bottom) {
      container.scrollTop += (elRect.bottom - containerRect.bottom + 4);
    }
  }, [tableSelectedIndex, tableOpen]);

  // Resize window — anchor top position, only grow downward
  useEffect(() => {
    (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const { LogicalSize, LogicalPosition } = await import("@tauri-apps/api/dpi");
        const win = getCurrentWindow();
        const scale = window.devicePixelRatio || 1;

        let targetHeight: number;
        if (previewData) {
          const maxH = window.screen.availHeight * 0.85;
          targetHeight = Math.min(500, maxH);
        } else if (showHelp) {
          targetHeight = 520;
        } else if (flatResults.length === 0 && !query.trim()) {
          targetHeight = 52;
        } else if (flatResults.length === 0) {
          targetHeight = 110;
        } else {
          const base = 52 + 44 + 22;
          const groupCost = leftGrouped.length * 38;
          const resultCost = flatResults.length * 44;
          targetHeight = base + groupCost + resultCost;
          const maxH = window.screen.availHeight * 0.85;
          targetHeight = Math.min(targetHeight, maxH);
        }

        // Set width based on whether table is open
        let targetWidth = 600;
        if (tableOpen) {
          const maxW = window.screen.availWidth * 0.85 / scale;
          targetWidth = Math.min(1200, maxW);
          targetHeight = Math.max(targetHeight, 400); // table needs minimum height

          // Save position if not already saved, and shift left if needed
          if (!originalWindowPos) {
            const pos = await win.outerPosition();
            const logicalX = pos.x / scale;
            const logicalY = pos.y / scale;
            setOriginalWindowPos({ x: logicalX, y: logicalY });

            const screenW = window.screen.availWidth / scale;
            if (logicalX + targetWidth > screenW) {
              const newX = Math.max(0, screenW - targetWidth);
              await win.setPosition(new LogicalPosition(newX, logicalY));
            }
          }
        }

        await win.setSize(new LogicalSize(targetWidth, targetHeight));
      } catch (e) {
        console.error(`[Omni resize] ERROR:`, e);
      }
    })();
  }, [flatResults.length, leftGrouped.length, query, showHelp, previewData, tableOpen]);

  // Listen for backend events
  useEffect(() => {
    const unlistenClear = listen("clear-query", () => {
      setQuery("");
      setResults([]);
      setSelectedIndex(0);
      setPreviewData(null);
      setMultiSelected(new Set());
      setTableOpen(false);
      setActivePanel("results");
      setTableResults([]);
      setTableMultiSelected(new Set());
    });
    const unlistenShown = listen("window-shown", async () => {
      invoke("refresh_apps");
      try {
        const frequent = await invoke<SearchResult[]>("get_frequent_items");
        if (frequent.length > 0) {
          setResults(frequent);
        }
      } catch (e) {
        console.error("Frequent items error:", e);
      }
    });
    const unlistenSelect = listen("select-query", () => {
      // Alt+Space when already visible: select all text in search bar and focus it
      const input = document.querySelector(".omni-input") as HTMLInputElement | null;
      if (input) {
        input.select();
        input.focus();
      }
      setActivePanel("results");
    });
    return () => {
      unlistenClear.then((fn) => fn());
      unlistenShown.then((fn) => fn());
      unlistenSelect.then((fn) => fn());
    };
  }, []);

  let globalIndex = 0;
  const activeCategory = getSelectedCategory();

  return (
    <div class="omni-container">
      <SearchInput value={query} onInput={handleInput} onKeyDown={handleKeyDown} />
      {tableOpen ? (
        <div class="omni-split">
          <div class={`results-container ${activePanel === "results" ? "panel-active" : "panel-inactive"}`}>
            {leftGrouped.map((group) => {
              const startIndex = globalIndex;
              globalIndex += group.results.length;
              return (
                <ResultGroup
                  key={group.category}
                  category={group.category}
                  results={group.results}
                  selectedIndex={activePanel === "results" ? selectedIndex : -1}
                  globalStartIndex={startIndex}
                  onExecute={executeResult}
                  isActive={activePanel === "results" && group.category === activeCategory}
                  isExpanded={group.category === expandedCategory}
                  multiSelected={activePanel === "results" ? multiSelected : new Set()}
                />
              );
            })}
            {copiedFlash && <div class="copied-flash">Copied!</div>}
          </div>
          <div class="table-divider" />
          <div class={activePanel === "table" ? "panel-active" : "panel-inactive"} style={{ display: "flex", flex: 1, minWidth: 0 }}>
            {previewData ? (
              <div class="results-container" style={{ flex: 1 }}>
                <PreviewPanel preview={previewData} />
              </div>
            ) : contextMenuIndex === -2 && tableContextResult ? (
              <div class="results-container" style={{ flex: 1 }}>
                <ContextMenu
                  result={tableContextResult}
                  selectedAction={contextActionIndex}
                  onExecute={(actionIndex: number) => {
                    const actions = getActions(tableContextResult);
                    if (actionIndex >= 0 && actionIndex < actions.length) {
                      actions[actionIndex].handler();
                    }
                    setContextMenuIndex(null);
                    setContextActionIndex(0);
                    setTableContextResult(null);
                  }}
                />
              </div>
            ) : (
              <TablePanel
                results={tableResults}
                selectedIndex={tableSelectedIndex}
                multiSelected={tableMultiSelected}
                onSelect={(i) => { setTableSelectedIndex(i); setActivePanel("table"); }}
                onExecute={(i) => {
                  const tr = tableResults[i];
                  if (tr) {
                    invoke("execute_action", { action: tr.action });
                    invoke("record_selection", { query, resultPath: tr.subtitle, category: tr.category, title: tr.title });
                  }
                }}
                sortColumn={tableSortColumn}
                sortAscending={tableSortAscending}
                onSortChange={(col, asc) => { setTableSortColumn(col); setTableSortAscending(asc); fetchTableResults(query, col, asc); }}
              />
            )}
          </div>
        </div>
      ) : previewData ? (
        <div class="results-container">
          <PreviewPanel preview={previewData} />
        </div>
      ) : contextMenuIndex !== null && contextMenuIndex === -1 ? (
        <div class="results-container">
          <BatchContextMenu
            count={multiSelected.size + 1}
            selectedAction={contextActionIndex}
            onExecute={(actionIndex: number) => {
              const indices = [...new Set([...multiSelected, selectedIndex])];
              const paths = indices.map(i => flatResults[i]?.subtitle).filter(Boolean);
              const batchActions = [
                { label: `Open all (${indices.length} items)`, handler: () => invoke("batch_open", { paths }) },
                { label: "Copy all paths", handler: () => navigator.clipboard.writeText(paths.join("\n")) },
                { label: "Copy all to...", handler: () => invoke("batch_copy_to", { paths }) },
                { label: "Move all to...", handler: () => invoke("batch_move_to", { paths }) },
                { label: "Delete all (recycle bin)", handler: async () => {
                  const confirmed = window.confirm(`Move ${paths.length} items to recycle bin?`);
                  if (confirmed) await invoke("batch_delete", { paths });
                }},
              ];
              if (actionIndex >= 0 && actionIndex < batchActions.length) {
                batchActions[actionIndex].handler();
              }
              setContextMenuIndex(null);
              setContextActionIndex(0);
              setMultiSelected(new Set());
            }}
          />
        </div>
      ) : contextMenuIndex !== null && flatResults[contextMenuIndex] ? (
        <div class="results-container">
          <ContextMenu
            result={flatResults[contextMenuIndex]}
            selectedAction={contextActionIndex}
            onExecute={executeContextAction}
          />
        </div>
      ) : flatResults.length > 0 ? (
        <div class="results-container">
          {leftGrouped.map((group) => {
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
                multiSelected={multiSelected}
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
              <div class="help-row"><kbd>Shift+↑↓</kbd><span>Multi-select items</span></div>
            </div>
            <div class="help-section">
              <div class="help-section-title">Actions</div>
              <div class="help-row"><kbd>Enter</kbd><span>Open / execute</span></div>
              <div class="help-row"><kbd>Shift+→</kbd><span>Context menu</span></div>
              <div class="help-row"><kbd>Shift+←</kbd><span>Close context menu</span></div>
              <div class="help-row"><kbd>Ctrl+C</kbd><span>Copy path of selected result</span></div>
              <div class="help-row"><kbd>Ctrl+E</kbd><span>Expand category (50 results)</span></div>
              <div class="help-row"><kbd>Ctrl+Space</kbd><span>Preview file (↑↓ scroll, PgUp/Dn page)</span></div>
              <div class="help-row"><kbd>Escape</kbd><span>Collapse / hide</span></div>
              <div class="help-row"><kbd>Ctrl+T</kbd><span>Table view (file columns)</span></div>
              <div class="help-row"><kbd>Ctrl+1-4</kbd><span>Sort table by column</span></div>
              <div class="help-row"><kbd>Ctrl+H</kbd><span>Toggle this help</span></div>
            </div>
          </div>
          <div class="help-section" style="margin-top: 10px;">
            <div class="help-section-title">Search Syntax</div>
            <div class="help-row"><kbd>C:\path\</kbd><span>Scope search to a directory</span></div>
            <div class="help-row"><kbd>F:\docs\report</kbd><span>Search "report" in F:\docs\</span></div>
            <div class="help-row"><kbd>*.rs</kbd><span>Wildcard extension match</span></div>
            <div class="help-row"><kbd>foo | bar</kbd><span>OR — match either term</span></div>
            <div class="help-row"><kbd>!node_modules</kbd><span>NOT — exclude term</span></div>
            <div class="help-row"><kbd>foo bar</kbd><span>Fuzzy fragments (matches *foo*bar*)</span></div>
            <div class="help-row"><kbd>regex:pattern</kbd><span>Regex search (also r:)</span></div>
          </div>
        </div>
      )}
      {flatResults.length > 0 && !showHelp && !previewData && contextMenuIndex === null && (
        <div class="status-bar">
          {tableOpen && activePanel === "table"
            ? (tableMultiSelected.size > 0
                ? `${tableMultiSelected.size + 1} items selected \u00b7 Shift+\u2192 for batch actions`
                : tableResults[tableSelectedIndex]?.subtitle || "")
            : multiSelected.size > 0
              ? `${multiSelected.size + 1} items selected \u00b7 Shift+\u2192 for batch actions`
              : flatResults[selectedIndex]?.subtitle || ""}
        </div>
      )}
    </div>
  );
}

import { useRef, useState } from "preact/hooks";

interface TableResult {
  category: string;
  title: string;
  subtitle: string;
  action: any;
  icon: string;
  size?: number;
  date_modified?: number;
}

type SortColumn = "name" | "type" | "path" | "size" | "date_modified";

const COLUMN_LABELS: Record<SortColumn, string> = {
  type: "Type",
  name: "Name",
  path: "Path",
  size: "Size",
  date_modified: "Modified",
};

const DEFAULT_COLUMN_ORDER: SortColumn[] = ["type", "name", "path", "size", "date_modified"];

interface Props {
  results: TableResult[];
  selectedIndex: number;
  multiSelected: Set<number>;
  sortColumn: SortColumn;
  sortAscending: boolean;
  columnOrder: SortColumn[];
  onSelect: (index: number) => void;
  onExecute: (index: number) => void;
  onSortChange: (column: SortColumn, ascending: boolean) => void;
  onColumnReorder: (newOrder: SortColumn[]) => void;
}

export function TablePanel({ results, selectedIndex, multiSelected, sortColumn, sortAscending, columnOrder, onSelect, onExecute, onSortChange, onColumnReorder }: Props) {
  const bodyRef = useRef<HTMLDivElement>(null);
  const [draggedCol, setDraggedCol] = useState<SortColumn | null>(null);
  const [dragOverCol, setDragOverCol] = useState<SortColumn | null>(null);

  const handleHeaderClick = (col: SortColumn) => {
    if (col === sortColumn) {
      onSortChange(col, !sortAscending);
    } else {
      onSortChange(col, col === "name" || col === "path" || col === "type");
    }
  };

  const sortIndicator = (col: SortColumn) => {
    if (col !== sortColumn) return null;
    return <span class="sort-arrow">{sortAscending ? "\u25B2" : "\u25BC"}</span>;
  };

  const handleDragStart = (col: SortColumn, e: DragEvent) => {
    setDraggedCol(col);
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", col);
    }
  };

  const handleDragOver = (col: SortColumn, e: DragEvent) => {
    e.preventDefault();
    if (col !== draggedCol) {
      setDragOverCol(col);
    }
  };

  const handleDrop = (col: SortColumn, e: DragEvent) => {
    e.preventDefault();
    if (draggedCol && draggedCol !== col) {
      const newOrder = [...columnOrder];
      const fromIdx = newOrder.indexOf(draggedCol);
      const toIdx = newOrder.indexOf(col);
      newOrder.splice(fromIdx, 1);
      newOrder.splice(toIdx, 0, draggedCol);
      onColumnReorder(newOrder);
    }
    setDraggedCol(null);
    setDragOverCol(null);
  };

  const handleDragEnd = () => {
    setDraggedCol(null);
    setDragOverCol(null);
  };

  const renderCell = (col: SortColumn, r: TableResult) => {
    switch (col) {
      case "type":
        return <div class="table-col col-type">{getExtension(r.title)}</div>;
      case "name":
        return (
          <div class="table-col col-name">
            <span class="table-icon">{getFileIcon(r.title)}</span>
            {r.title}
          </div>
        );
      case "path":
        return <div class="table-col col-path">{getParentPath(r.subtitle)}</div>;
      case "size":
        return <div class="table-col col-size">{formatSize(r.size)}</div>;
      case "date_modified":
        return <div class="table-col col-date">{formatDate(r.date_modified)}</div>;
    }
  };

  // Calculate visible rows for page indicator
  const visibleRows = bodyRef.current
    ? Math.floor(bodyRef.current.clientHeight / 27)
    : 20;
  const totalPages = visibleRows > 0 ? Math.ceil(results.length / visibleRows) : 1;
  const currentPage = visibleRows > 0
    ? Math.floor(selectedIndex / visibleRows) + 1
    : 1;

  return (
    <div class="table-panel">
      <div class="table-header">
        {columnOrder.map((col) => (
          <div
            key={col}
            class={`table-col col-${col === "date_modified" ? "date" : col} ${draggedCol === col ? "col-dragging" : ""} ${dragOverCol === col ? "col-drag-over" : ""}`}
            onClick={() => handleHeaderClick(col)}
            draggable
            onDragStart={(e) => handleDragStart(col, e as DragEvent)}
            onDragOver={(e) => handleDragOver(col, e as DragEvent)}
            onDrop={(e) => handleDrop(col, e as DragEvent)}
            onDragEnd={handleDragEnd}
          >
            {COLUMN_LABELS[col]} {sortIndicator(col)}
          </div>
        ))}
      </div>
      <div class="table-body" ref={bodyRef}>
        {results.length === 0 ? (
          <div class="table-empty">No file results</div>
        ) : (
          results.map((r, i) => (
            <div
              key={`${r.subtitle}-${i}`}
              class={`table-row ${i === selectedIndex ? "selected" : ""} ${multiSelected.has(i) ? "multi-selected" : ""}`}
              onClick={() => onSelect(i)}
              onDblClick={() => onExecute(i)}
            >
              {columnOrder.map((col) => renderCell(col, r))}
            </div>
          ))
        )}
      </div>
      {results.length > 0 && (
        <div class="table-footer">
          {results.length} items - page {currentPage}/{totalPages}
        </div>
      )}
    </div>
  );
}

function getFileIcon(filename: string): string {
  const ext = filename.split(".").pop()?.toLowerCase() || "";
  const iconMap: Record<string, string> = {
    exe: "\u2B21", pdf: "PDF", txt: "TXT", md: "MD",
    jpg: "IMG", jpeg: "IMG", png: "IMG", gif: "GIF", svg: "SVG",
    mp3: "\u266A", wav: "\u266A", mp4: "\u25B6",
    zip: "ZIP", rar: "RAR", "7z": "7Z",
    py: "PY", rs: "RS", js: "JS", ts: "TS",
    json: "{}", html: "<>", css: "#",
  };
  return iconMap[ext] || "\u25CF";
}

function getExtension(filename: string): string {
  const dot = filename.lastIndexOf(".");
  if (dot <= 0) return "\u2014";
  return filename.substring(dot + 1).toUpperCase();
}

function getParentPath(fullPath: string): string {
  const lastSep = fullPath.lastIndexOf("\\");
  return lastSep > 0 ? fullPath.substring(0, lastSep) : fullPath;
}

function formatSize(bytes?: number): string {
  if (bytes == null) return "\u2014";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function formatDate(epoch?: number): string {
  if (epoch == null) return "\u2014";
  const date = new Date(epoch * 1000);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays === 1) return "yesterday";
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toISOString().slice(0, 10);
}

export { DEFAULT_COLUMN_ORDER };
export type { TableResult, SortColumn };

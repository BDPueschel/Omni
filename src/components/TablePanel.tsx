import { useRef } from "preact/hooks";

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

interface Props {
  results: TableResult[];
  selectedIndex: number;
  multiSelected: Set<number>;
  sortColumn: SortColumn;
  sortAscending: boolean;
  onSelect: (index: number) => void;
  onExecute: (index: number) => void;
  onSortChange: (column: SortColumn, ascending: boolean) => void;
}

export function TablePanel({ results, selectedIndex, multiSelected, sortColumn, sortAscending, onSelect, onExecute, onSortChange }: Props) {
  const bodyRef = useRef<HTMLDivElement>(null);

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
        <div class="table-col col-type" onClick={() => handleHeaderClick("type")}>
          Type {sortIndicator("type")}
        </div>
        <div class="table-col col-name" onClick={() => handleHeaderClick("name")}>
          Name {sortIndicator("name")}
        </div>
        <div class="table-col col-path" onClick={() => handleHeaderClick("path")}>
          Path {sortIndicator("path")}
        </div>
        <div class="table-col col-size" onClick={() => handleHeaderClick("size")}>
          Size {sortIndicator("size")}
        </div>
        <div class="table-col col-date" onClick={() => handleHeaderClick("date_modified")}>
          Modified {sortIndicator("date_modified")}
        </div>
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
              <div class="table-col col-type">{getExtension(r.title)}</div>
              <div class="table-col col-name">
                <span class="table-icon">{getFileIcon(r.title)}</span>
                {r.title}
              </div>
              <div class="table-col col-path">{getParentPath(r.subtitle)}</div>
              <div class="table-col col-size">{formatSize(r.size)}</div>
              <div class="table-col col-date">{formatDate(r.date_modified)}</div>
            </div>
          ))
        )}
      </div>
      {results.length > 0 && (
        <div class="table-footer">
          {results.length} items \u00b7 page {currentPage}/{totalPages}
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

export type { TableResult, SortColumn };

interface SearchResult {
  category: string;
  title: string;
  subtitle: string;
  action: any;
  icon: string;
}

interface Props {
  result: SearchResult;
  isSelected: boolean;
  onExecute: () => void;
}

export function ResultItem({ result, isSelected, onExecute }: Props) {
  const iconInfo = getIcon(result.icon, result.subtitle);

  return (
    <div
      class={`result-item ${isSelected ? "selected" : ""}`}
      onClick={onExecute}
    >
      <div class="result-icon" style={{ color: iconInfo.color }}>
        {iconInfo.symbol}
      </div>
      <div class="result-text">
        <div class="result-title">{result.title}</div>
        <div class="result-subtitle">{result.subtitle}</div>
      </div>
    </div>
  );
}

interface IconInfo {
  symbol: string;
  color: string;
}

function getIcon(icon: string, subtitle: string): IconInfo {
  const dim = "rgba(255,255,255,0.5)";

  // File-type icons based on extension
  if (icon === "file" && subtitle) {
    const ext = subtitle.split('.').pop()?.toLowerCase() || "";
    const extMap: Record<string, IconInfo> = {
      exe:  { symbol: "⬡", color: "#6fc3df" },
      lnk:  { symbol: "🔗", color: dim },
      pdf:  { symbol: "PDF", color: "#e74c3c" },
      txt:  { symbol: "TXT", color: dim },
      md:   { symbol: "MD", color: dim },
      log:  { symbol: "LOG", color: dim },
      jpg:  { symbol: "IMG", color: "#e67e22" },
      jpeg: { symbol: "IMG", color: "#e67e22" },
      png:  { symbol: "IMG", color: "#e67e22" },
      gif:  { symbol: "GIF", color: "#e67e22" },
      svg:  { symbol: "SVG", color: "#e67e22" },
      mp3:  { symbol: "♪", color: "#9b59b6" },
      wav:  { symbol: "♪", color: "#9b59b6" },
      flac: { symbol: "♪", color: "#9b59b6" },
      mp4:  { symbol: "▶", color: "#e74c3c" },
      avi:  { symbol: "▶", color: "#e74c3c" },
      mkv:  { symbol: "▶", color: "#e74c3c" },
      zip:  { symbol: "ZIP", color: "#f39c12" },
      rar:  { symbol: "RAR", color: "#f39c12" },
      "7z": { symbol: "7Z", color: "#f39c12" },
      py:   { symbol: "PY", color: "#3498db" },
      rs:   { symbol: "RS", color: "#e67e22" },
      js:   { symbol: "JS", color: "#f1c40f" },
      ts:   { symbol: "TS", color: "#3498db" },
      json: { symbol: "{}", color: "#95a5a6" },
      html: { symbol: "<>", color: "#e67e22" },
      css:  { symbol: "#", color: "#3498db" },
      toml: { symbol: "CFG", color: "#95a5a6" },
      yaml: { symbol: "YML", color: "#95a5a6" },
      yml:  { symbol: "YML", color: "#95a5a6" },
    };
    if (extMap[ext]) return extMap[ext];
    return { symbol: "●", color: dim };
  }

  // Category icons
  const iconMap: Record<string, IconInfo> = {
    calculator: { symbol: "=", color: "#2ecc71" },
    app:        { symbol: "◆", color: "#6fc3df" },
    folder:     { symbol: "📁", color: "#f39c12" },
    file:       { symbol: "●", color: dim },
    search:     { symbol: "⌕", color: dim },
    globe:      { symbol: "⊕", color: "#3498db" },
    lock:       { symbol: "🔒", color: "#f39c12" },
    moon:       { symbol: "☽", color: "#9b59b6" },
    power:      { symbol: "⏻", color: "#e74c3c" },
    refresh:    { symbol: "↻", color: "#3498db" },
    trash:      { symbol: "♻", color: "#95a5a6" },
    "log-out":  { symbol: "→", color: "#e67e22" },
    alert:      { symbol: "⚠", color: "#f39c12" },
    unit:       { symbol: "↔", color: "#2ecc71" },
    currency:   { symbol: "$", color: "#f1c40f" },
  };

  return iconMap[icon] || { symbol: "•", color: dim };
}

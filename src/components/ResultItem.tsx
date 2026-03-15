import { useState, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";

// Module-level cache for icon data URIs
const iconCache = new Map<string, string>();

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
  const [iconUri, setIconUri] = useState<string | null>(null);

  useEffect(() => {
    if (result.icon === "app" && result.subtitle) {
      const cached = iconCache.get(result.subtitle);
      if (cached) {
        setIconUri(cached);
      } else {
        invoke<string>("get_icon", { path: result.subtitle }).then((uri) => {
          if (uri) {
            iconCache.set(result.subtitle, uri);
            setIconUri(uri);
          }
        });
      }
    }
  }, [result.subtitle, result.icon]);

  const isColorSwatch = result.icon.startsWith("color:");
  const colorHex = isColorSwatch ? result.icon.slice(6) : null;

  return (
    <div
      class={`result-item ${isSelected ? "selected" : ""}`}
      onClick={onExecute}
    >
      {isColorSwatch ? (
        <div class="result-icon" style={{ background: colorHex!, borderRadius: '4px' }}></div>
      ) : (
        <div class="result-icon" style={{ color: iconInfo.color }}>
          {iconUri ? (
            <img src={iconUri} class="result-icon-img" />
          ) : (
            iconInfo.symbol
          )}
        </div>
      )}
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
      exe:  { symbol: "\u2B21", color: "#6fc3df" },
      lnk:  { symbol: "\uD83D\uDD17", color: dim },
      pdf:  { symbol: "PDF", color: "#e74c3c" },
      txt:  { symbol: "TXT", color: dim },
      md:   { symbol: "MD", color: dim },
      log:  { symbol: "LOG", color: dim },
      jpg:  { symbol: "IMG", color: "#e67e22" },
      jpeg: { symbol: "IMG", color: "#e67e22" },
      png:  { symbol: "IMG", color: "#e67e22" },
      gif:  { symbol: "GIF", color: "#e67e22" },
      svg:  { symbol: "SVG", color: "#e67e22" },
      mp3:  { symbol: "\u266A", color: "#9b59b6" },
      wav:  { symbol: "\u266A", color: "#9b59b6" },
      flac: { symbol: "\u266A", color: "#9b59b6" },
      mp4:  { symbol: "\u25B6", color: "#e74c3c" },
      avi:  { symbol: "\u25B6", color: "#e74c3c" },
      mkv:  { symbol: "\u25B6", color: "#e74c3c" },
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
    return { symbol: "\u25CF", color: dim };
  }

  // Category icons
  const iconMap: Record<string, IconInfo> = {
    calculator: { symbol: "=", color: "#2ecc71" },
    app:        { symbol: "\u25C6", color: "#6fc3df" },
    folder:     { symbol: "\uD83D\uDCC1", color: "#f39c12" },
    file:       { symbol: "\u25CF", color: dim },
    search:     { symbol: "\u2315", color: dim },
    globe:      { symbol: "\u2295", color: "#3498db" },
    lock:       { symbol: "\uD83D\uDD12", color: "#f39c12" },
    moon:       { symbol: "\u263D", color: "#9b59b6" },
    power:      { symbol: "\u23FB", color: "#e74c3c" },
    refresh:    { symbol: "\u21BB", color: "#3498db" },
    trash:      { symbol: "\u267B", color: "#95a5a6" },
    "log-out":  { symbol: "\u2192", color: "#e67e22" },
    alert:      { symbol: "\u26A0", color: "#f39c12" },
    unit:       { symbol: "\u2194", color: "#2ecc71" },
    currency:   { symbol: "$", color: "#f1c40f" },
    process:    { symbol: "\u00D7", color: "#e74c3c" },
    clipboard:  { symbol: "CB", color: "#9b59b6" },
  };

  return iconMap[icon] || { symbol: "\u2022", color: dim };
}

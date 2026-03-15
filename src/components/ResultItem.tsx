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
  return (
    <div
      class={`result-item ${isSelected ? "selected" : ""}`}
      onClick={onExecute}
    >
      <div class="result-icon">{getIcon(result.icon, result.subtitle)}</div>
      <div class="result-text">
        <div class="result-title">{result.title}</div>
        <div class="result-subtitle">{result.subtitle}</div>
      </div>
    </div>
  );
}

function getIcon(icon: string, subtitle: string): string {
  // File-type specific icons based on extension in subtitle path
  if (icon === "file" && subtitle) {
    const ext = subtitle.split('.').pop()?.toLowerCase();
    const extIcons: Record<string, string> = {
      exe: "\u{1F4E6}",
      lnk: "\u{1F517}",
      pdf: "\u{1F4D1}",
      txt: "\u{1F4DD}",
      md: "\u{1F4DD}",
      jpg: "\u{1F5BC}",
      jpeg: "\u{1F5BC}",
      png: "\u{1F5BC}",
      gif: "\u{1F5BC}",
      mp3: "\u{1F3B5}",
      wav: "\u{1F3B5}",
      mp4: "\u{1F3AC}",
      avi: "\u{1F3AC}",
      zip: "\u{1F4E6}",
      rar: "\u{1F4E6}",
      py: "\u{1F40D}",
      rs: "\u{2699}",
      js: "\u{1F7E8}",
      ts: "\u{1F7E6}",
      json: "\u{1F4CB}",
      html: "\u{1F310}",
      css: "\u{1F3A8}",
    };
    if (ext && extIcons[ext]) return extIcons[ext];
  }

  const icons: Record<string, string> = {
    calculator: "\u{1F9EE}",
    app: "\u{25B6}",
    file: "\u{1F4C4}",
    search: "\u{1F50D}",
    globe: "\u{1F310}",
    lock: "\u{1F512}",
    moon: "\u{1F319}",
    power: "\u{23FB}",
    refresh: "\u{1F504}",
    trash: "\u{1F5D1}",
    "log-out": "\u{1F6AA}",
    alert: "\u{26A0}",
  };
  return icons[icon] || "\u{2022}";
}

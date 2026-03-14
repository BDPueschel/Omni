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
      <div class="result-icon">{getIcon(result.icon)}</div>
      <div class="result-text">
        <div class="result-title">{result.title}</div>
        <div class="result-subtitle">{result.subtitle}</div>
      </div>
    </div>
  );
}

function getIcon(icon: string): string {
  const icons: Record<string, string> = {
    calculator: "\u{1F5A9}",
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

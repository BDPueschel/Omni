import { ResultItem } from "./ResultItem";

interface SearchResult {
  category: string;
  title: string;
  subtitle: string;
  action: any;
  icon: string;
}

interface Props {
  category: string;
  results: SearchResult[];
  selectedIndex: number;
  globalStartIndex: number;
  onExecute: (index: number) => void;
  isActive: boolean;
  isExpanded: boolean;
  multiSelected: Set<number>;
}

const EXPANDABLE = ["Apps", "Files", "Directories", "System"];

export function ResultGroup({ category, results, selectedIndex, globalStartIndex, onExecute, isActive, isExpanded, multiSelected }: Props) {
  if (results.length === 0) return null;

  const canExpand = EXPANDABLE.includes(category) && !isExpanded;
  const hint = isExpanded ? "expanded" : canExpand && isActive ? "Ctrl+E to expand" : "";

  return (
    <div class={`result-group ${isActive ? "active-group" : ""}`}>
      <div class="result-group-header">
        <span>{category} ({results.length})</span>
        {hint && <span class="group-hint">{hint}</span>}
      </div>
      {results.map((result, i) => (
        <ResultItem
          key={`${category}-${i}`}
          result={result}
          isSelected={selectedIndex === globalStartIndex + i}
          isMultiSelected={multiSelected.has(globalStartIndex + i)}
          onExecute={() => onExecute(globalStartIndex + i)}
        />
      ))}
    </div>
  );
}

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
}

export function ResultGroup({ category, results, selectedIndex, globalStartIndex, onExecute }: Props) {
  if (results.length === 0) return null;

  return (
    <div class="result-group">
      <div class="result-group-header">{category}</div>
      {results.map((result, i) => (
        <ResultItem
          key={`${category}-${i}`}
          result={result}
          isSelected={selectedIndex === globalStartIndex + i}
          onExecute={() => onExecute(globalStartIndex + i)}
        />
      ))}
    </div>
  );
}

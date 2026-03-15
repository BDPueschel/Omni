import { invoke } from "@tauri-apps/api/core";

interface SearchResult {
  category: string;
  title: string;
  subtitle: string;
  action: any;
  icon: string;
}

interface ContextAction {
  label: string;
  shortcut?: string;
  handler: () => Promise<void>;
}

interface Props {
  result: SearchResult;
  selectedAction: number;
  onExecute: (index: number) => void;
}

function getActions(result: SearchResult): ContextAction[] {
  const path = result.subtitle;
  const actions: ContextAction[] = [];

  if (result.category === "Files" || result.category === "Directories" || result.category === "Apps") {
    actions.push({
      label: "Open containing folder",
      handler: async () => { await invoke("open_containing_folder", { path }); },
    });
    actions.push({
      label: "Copy path",
      handler: async () => { await navigator.clipboard.writeText(path); },
    });
    actions.push({
      label: "Open in terminal",
      handler: async () => { await invoke("open_in_terminal", { path }); },
    });
    actions.push({
      label: "Open in VS Code",
      handler: async () => { await invoke("open_in_vscode", { path }); },
    });
  }

  if (result.category === "Apps") {
    actions.push({
      label: "Run as administrator",
      handler: async () => { await invoke("run_as_admin", { path }); },
    });
  }

  return actions;
}

export function ContextMenu({ result, selectedAction, onExecute }: Props) {
  const actions = getActions(result);

  if (actions.length === 0) return null;

  return (
    <div class="context-menu">
      <div class="context-menu-header">
        {result.title}
        <span class="context-hint">← back</span>
      </div>
      {actions.map((action, i) => (
        <div
          key={i}
          class={`context-action ${i === selectedAction ? "selected" : ""}`}
          onClick={() => onExecute(i)}
        >
          {action.label}
        </div>
      ))}
    </div>
  );
}

export { getActions };
export type { ContextAction };

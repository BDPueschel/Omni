import { useState, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";

interface Config {
  hotkey: string;
  max_results_per_category: number;
  search_engine: string;
  start_with_windows: boolean;
  theme_opacity: number;
  use_system_accent: boolean;
}

interface Props {
  visible: boolean;
  onDismiss: () => void;
}

export function SettingsPanel({ visible, onDismiss }: Props) {
  const [config, setConfig] = useState<Config | null>(null);

  useEffect(() => {
    if (visible) {
      invoke<Config>("get_config").then(setConfig);
    }
  }, [visible]);

  if (!visible || !config) return null;

  const update = async (key: keyof Config, value: any) => {
    const updated = { ...config, [key]: value };
    setConfig(updated);
    await invoke("save_config", { config: updated });
    if (key === "use_system_accent") {
      await invoke("update_tray_icon");
    }
  };

  return (
    <div class="settings-backdrop" onClick={onDismiss}>
      <div class="settings-panel" onClick={(e) => e.stopPropagation()}>
        <div class="settings-header">
          <span>Settings</span>
          <span class="settings-dismiss" onClick={onDismiss}>Esc</span>
        </div>

        <div class="settings-body">
          <div class="setting-row">
            <label>Accent color</label>
            <select
              value={config.use_system_accent ? "system" : "blue"}
              onChange={(e) => update("use_system_accent", (e.target as HTMLSelectElement).value === "system")}
            >
              <option value="blue">Omni Blue</option>
              <option value="system">Windows Accent</option>
            </select>
          </div>

          <div class="setting-row">
            <label>Search engine</label>
            <select
              value={config.search_engine}
              onChange={(e) => update("search_engine", (e.target as HTMLSelectElement).value)}
            >
              <option value="google">Google</option>
              <option value="duckduckgo">DuckDuckGo</option>
            </select>
          </div>

          <div class="setting-row">
            <label>Max results per category</label>
            <input
              type="number"
              min="1"
              max="20"
              value={config.max_results_per_category}
              onInput={(e) => update("max_results_per_category", parseInt((e.target as HTMLInputElement).value) || 5)}
            />
          </div>

          <div class="setting-row">
            <label>Window opacity ({config.theme_opacity}%)</label>
            <input
              type="range"
              min="40"
              max="100"
              value={config.theme_opacity}
              onInput={(e) => update("theme_opacity", parseInt((e.target as HTMLInputElement).value))}
            />
          </div>

          <div class="setting-row">
            <label>Start with Windows</label>
            <input
              type="checkbox"
              checked={config.start_with_windows}
              onChange={(e) => update("start_with_windows", (e.target as HTMLInputElement).checked)}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

import { render } from "preact";
import { useState, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import "./settings.css";

interface Config {
  hotkey: string;
  max_results_per_category: number;
  search_engine: string;
  start_with_windows: boolean;
  theme_opacity: number;
  use_system_accent: boolean;
}

function Settings() {
  const [config, setConfig] = useState<Config | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<Config>("get_config").then(setConfig);
  }, []);

  if (!config) return <div class="settings-loading">Loading...</div>;

  const update = (key: keyof Config, value: any) => {
    setConfig({ ...config, [key]: value });
    setSaved(false);
  };

  const save = async () => {
    await invoke("save_config", { config });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div class="settings-container">
      <h1>Omni Settings</h1>

      <div class="setting-row">
        <label>Hotkey</label>
        <input type="text" value={config.hotkey}
          onInput={(e) => update("hotkey", (e.target as HTMLInputElement).value)} />
      </div>

      <div class="setting-row">
        <label>Max results per category</label>
        <input type="number" min="1" max="20" value={config.max_results_per_category}
          onInput={(e) => update("max_results_per_category", parseInt((e.target as HTMLInputElement).value) || 5)} />
      </div>

      <div class="setting-row">
        <label>Web search engine</label>
        <select value={config.search_engine}
          onChange={(e) => update("search_engine", (e.target as HTMLSelectElement).value)}>
          <option value="google">Google</option>
          <option value="duckduckgo">DuckDuckGo</option>
        </select>
      </div>

      <div class="setting-row">
        <label>Start with Windows</label>
        <input type="checkbox" checked={config.start_with_windows}
          onChange={(e) => update("start_with_windows", (e.target as HTMLInputElement).checked)} />
      </div>

      <div class="setting-row">
        <label>Theme opacity ({config.theme_opacity}%)</label>
        <input type="range" min="40" max="100" value={config.theme_opacity}
          onInput={(e) => update("theme_opacity", parseInt((e.target as HTMLInputElement).value))} />
      </div>

      <div class="setting-row">
        <label>Accent color</label>
        <select value={config.use_system_accent ? "system" : "blue"}
          onChange={(e) => update("use_system_accent", (e.target as HTMLSelectElement).value === "system")}>
          <option value="blue">Omni Blue</option>
          <option value="system">Windows System Accent</option>
        </select>
      </div>

      <button class="save-button" onClick={save}>
        {saved ? "Saved!" : "Save"}
      </button>
    </div>
  );
}

render(<Settings />, document.getElementById("settings-app")!);

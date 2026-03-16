import { useRef, useEffect, useState } from "preact/hooks";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  value: string;
  onInput: (value: string) => void;
  onKeyDown: (e: KeyboardEvent) => void;
}

/** Hue-shift an image to a target RGB accent color via an offscreen canvas. */
function recolorIcon(src: string, r: number, g: number, b: number): Promise<string> {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = img.width;
      canvas.height = img.height;
      const ctx = canvas.getContext("2d")!;
      ctx.drawImage(img, 0, 0);
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const data = imageData.data;

      // Convert target accent to HSL to get the target hue
      const targetHsl = rgbToHsl(r, g, b);
      const baseHue = 220; // the icon's blue hue
      const hueShift = targetHsl[0] - baseHue;
      const satRatio = targetHsl[1] > 0.01 ? targetHsl[1] / 0.7 : 1.0;

      for (let i = 0; i < data.length; i += 4) {
        if (data[i + 3] === 0) continue;
        const [h, s, l] = rgbToHsl(data[i], data[i + 1], data[i + 2]);
        if (s < 0.05) continue; // skip greys
        let newH = (h + hueShift) % 360;
        if (newH < 0) newH += 360;
        const newS = Math.min(1, Math.max(0, s * satRatio));
        const [nr, ng, nb] = hslToRgb(newH, newS, l);
        data[i] = nr;
        data[i + 1] = ng;
        data[i + 2] = nb;
      }

      ctx.putImageData(imageData, 0, 0);
      resolve(canvas.toDataURL("image/png"));
    };
    img.src = src;
  });
}

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  const l = (max + min) / 2;
  if (max - min < 1e-10) return [0, 0, l];
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h: number;
  if (max === r) { h = (g - b) / d + (g < b ? 6 : 0); }
  else if (max === g) { h = (b - r) / d + 2; }
  else { h = (r - g) / d + 4; }
  return [h * 60, s, l];
}

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  if (s < 1e-10) { const v = Math.round(l * 255); return [v, v, v]; }
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
  const p = 2 * l - q;
  const hN = h / 360;
  const hue2rgb = (t: number) => {
    if (t < 0) t += 1; if (t > 1) t -= 1;
    if (t < 1/6) return p + (q - p) * 6 * t;
    if (t < 1/2) return q;
    if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
    return p;
  };
  return [
    Math.round(hue2rgb(hN + 1/3) * 255),
    Math.round(hue2rgb(hN) * 255),
    Math.round(hue2rgb(hN - 1/3) * 255),
  ];
}

export function SearchInput({ value, onInput, onKeyDown }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [iconSrc, setIconSrc] = useState("/omni-icon.png");

  // Apply accent-colored icon
  useEffect(() => {
    (async () => {
      try {
        const config = await invoke<{ use_system_accent?: boolean }>("get_config");
        if (config.use_system_accent) {
          const [r, g, b] = await invoke<[number, number, number]>("get_system_accent");
          const recolored = await recolorIcon("/omni-icon.png", r, g, b);
          setIconSrc(recolored);
        } else {
          setIconSrc("/omni-icon.png");
        }
      } catch (e) {
        console.error("Icon recolor error:", e);
      }
    })();
  }, []);

  // Focus on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Re-focus when window is shown via Alt+Space
  useEffect(() => {
    const unlisten = listen("window-shown", async () => {
      inputRef.current?.focus();
      requestAnimationFrame(() => inputRef.current?.focus());
      // Re-apply icon in case accent setting changed
      try {
        const config = await invoke<{ use_system_accent?: boolean }>("get_config");
        if (config.use_system_accent) {
          const [r, g, b] = await invoke<[number, number, number]>("get_system_accent");
          const recolored = await recolorIcon("/omni-icon.png", r, g, b);
          setIconSrc(recolored);
        } else {
          setIconSrc("/omni-icon.png");
        }
      } catch (_) {}
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  return (
    <div class="search-input-wrapper">
      <img src={iconSrc} class="search-icon-img" />
      <input
        ref={inputRef}
        type="text"
        class="omni-input"
        autoFocus
        placeholder="Search files, apps, or type a command..."
        value={value}
        onInput={(e) => onInput((e.target as HTMLInputElement).value)}
        onKeyDown={onKeyDown}
      />
    </div>
  );
}

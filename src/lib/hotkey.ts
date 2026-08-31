/** OS-aware hotkey presets and KeyboardEvent → settings string. */

export type HotkeyOs = "windows" | "macos" | "linux";

export function detectHotkeyOs(): HotkeyOs {
  const p = navigator.platform.toLowerCase();
  const ua = navigator.userAgent.toLowerCase();
  if (p.includes("mac") || ua.includes("mac os")) return "macos";
  if (p.includes("win") || ua.includes("windows")) return "windows";
  return "linux";
}

/** Canonical storage form used by Rust parse_hotkey. */
export function formatHotkeyFromEvent(e: KeyboardEvent): string | null {
  if (isModifierCode(e.code)) return null;

  const key = codeToKeyPart(e.code);
  if (!key) return null;

  const parts: string[] = [];
  const os = detectHotkeyOs();

  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push(os === "macos" ? "Cmd" : "Win");

  // Не берём одиночные буквы и голый Space — иначе Space случайно затирает хоткей.
  // Голые F1–F12 оставляем.
  if (parts.length === 0 && !key.startsWith("F")) {
    return null;
  }

  parts.push(key);
  return parts.join("+");
}

export function displayHotkey(hotkey: string, os: HotkeyOs = detectHotkeyOs()): string {
  if (os === "macos") {
    return hotkey
      .replace(/\bCtrl\b/gi, "⌃")
      .replace(/\bAlt\b/gi, "⌥")
      .replace(/\bShift\b/gi, "⇧")
      .replace(/\bCmd\b/gi, "⌘")
      .replace(/\bWin\b/gi, "⌘")
      .replace(/\+/g, "");
  }
  return hotkey.replace(/\bCmd\b/gi, "Win");
}

function isModifierCode(code: string): boolean {
  return (
    code === "ControlLeft" ||
    code === "ControlRight" ||
    code === "ShiftLeft" ||
    code === "ShiftRight" ||
    code === "AltLeft" ||
    code === "AltRight" ||
    code === "MetaLeft" ||
    code === "MetaRight" ||
    code === "OSLeft" ||
    code === "OSRight"
  );
}

export { isModifierCode };

function codeToKeyPart(code: string): string | null {
  if (code.startsWith("Key") && code.length === 4) {
    return code.slice(3);
  }
  if (code.startsWith("Digit") && code.length === 6) {
    return code.slice(5);
  }
  if (/^F([1-9]|1[0-2])$/.test(code)) {
    return code;
  }
  switch (code) {
    case "Space":
      return "Space";
    case "Enter":
      return "Enter";
    case "Tab":
      return "Tab";
    case "Escape":
      return "Esc";
    default:
      return null;
  }
}

export interface HotkeyPreset {
  value: string;
  label: string;
}

export function hotkeyPresetsForOs(os: HotkeyOs = detectHotkeyOs()): HotkeyPreset[] {
  if (os === "macos") {
    return [
      { value: "Cmd+Shift+T", label: "⌘⇧T" },
      { value: "Cmd+Shift+Space", label: "⌘⇧Space" },
      { value: "Cmd+Alt+T", label: "⌘⌥T" },
      { value: "Ctrl+Shift+T", label: "⌃⇧T" },
      { value: "Cmd+Shift+N", label: "⌘⇧N" },
      { value: "F9", label: "F9" },
      { value: "Cmd+F9", label: "⌘F9" },
    ];
  }

  // Windows & Linux — avoid Win+ alone (OS reserved); prefer Ctrl/Alt combos
  return [
    { value: "Ctrl+Shift+T", label: "Ctrl+Shift+T" },
    { value: "Ctrl+Shift+Space", label: "Ctrl+Shift+Space" },
    { value: "Ctrl+Alt+T", label: "Ctrl+Alt+T" },
    { value: "Alt+Shift+T", label: "Alt+Shift+T" },
    { value: "Ctrl+Shift+N", label: "Ctrl+Shift+N" },
    { value: "F9", label: "F9" },
    { value: "Ctrl+F9", label: "Ctrl+F9" },
  ];
}

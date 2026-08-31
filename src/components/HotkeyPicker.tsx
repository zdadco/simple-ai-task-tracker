import { useEffect, useRef, useState } from "react";
import { applyHotkey, registerHotkey, unregisterHotkey } from "../lib/tauri";
import {
  detectHotkeyOs,
  displayHotkey,
  formatHotkeyFromEvent,
  hotkeyPresetsForOs,
  isModifierCode,
} from "../lib/hotkey";

interface HotkeyPickerProps {
  value: string;
  onChange: (hotkey: string) => void;
}

export default function HotkeyPicker({ value, onChange }: HotkeyPickerProps) {
  const os = detectHotkeyOs();
  const presets = hotkeyPresetsForOs(os);
  const [recording, setRecording] = useState(false);
  const [preview, setPreview] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const captureRef = useRef<HTMLDivElement>(null);
  const chordRef = useRef<string | null>(null);
  const mainKeyCodeRef = useRef<string | null>(null);
  const finishingRef = useRef(false);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const valueRef = useRef(value);
  valueRef.current = value;

  useEffect(() => {
    if (!recording) return;

    finishingRef.current = false;
    chordRef.current = null;
    mainKeyCodeRef.current = null;
    setPreview(null);
    setError(null);

    void unregisterHotkey().catch(() => {});

    // Не оставляем фокус на кнопках — Space иначе «нажимает» Отмена/Записать.
    const t = window.setTimeout(() => {
      captureRef.current?.focus();
    }, 0);

    async function finishRecording(chord: string | null) {
      if (finishingRef.current) return;
      finishingRef.current = true;
      setRecording(false);
      setPreview(null);
      chordRef.current = null;
      mainKeyCodeRef.current = null;

      if (chord) {
        try {
          await applyHotkey(chord);
          onChangeRef.current(chord);
          setError(null);
        } catch (e) {
          setError(`Не удалось назначить: ${e}`);
          try {
            await applyHotkey(valueRef.current);
          } catch {
            await registerHotkey().catch(() => {});
          }
        }
        return;
      }

      try {
        await applyHotkey(valueRef.current);
      } catch {
        await registerHotkey().catch(() => {});
      }
    }

    function onKeyDown(e: KeyboardEvent) {
      if (finishingRef.current) return;
      e.preventDefault();
      e.stopPropagation();
      if (e.repeat) return;

      if (e.code === "Escape") {
        void finishRecording(null);
        return;
      }

      if (isModifierCode(e.code)) {
        return;
      }

      // ОС-хоткей = модификаторы + одна клавиша. Первая основная не перезаписывается
      // (иначе Ctrl+Space, затем G превращается в Ctrl+G).
      if (mainKeyCodeRef.current) {
        return;
      }

      const formatted = formatHotkeyFromEvent(e);
      if (formatted) {
        mainKeyCodeRef.current = e.code;
        chordRef.current = formatted;
        setPreview(formatted);
        setError(null);
      } else if (e.code === "Space") {
        setError("Space только вместе с Ctrl / Alt / Shift (например Ctrl+Shift+Space)");
      }
    }

    function onKeyUp(e: KeyboardEvent) {
      if (finishingRef.current) return;
      e.preventDefault();
      e.stopPropagation();

      if (
        mainKeyCodeRef.current &&
        e.code === mainKeyCodeRef.current &&
        chordRef.current
      ) {
        void finishRecording(chordRef.current);
      }
    }

    function onMouseDown(e: MouseEvent) {
      if (finishingRef.current) return;
      // 0=left, 1=middle, 2=right, 3/4 = боковые
      if (e.button >= 3) {
        e.preventDefault();
        e.stopPropagation();
        setError(
          "Боковые кнопки мыши нельзя назначить: глобальный хоткей в ОС поддерживает только клавиатуру.",
        );
        setPreview(null);
        chordRef.current = null;
      }
    }

    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("keyup", onKeyUp, true);
    window.addEventListener("mousedown", onMouseDown, true);

    return () => {
      window.clearTimeout(t);
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("keyup", onKeyUp, true);
      window.removeEventListener("mousedown", onMouseDown, true);
    };
  }, [recording]);

  async function cancelRecording() {
    finishingRef.current = true;
    setRecording(false);
    setPreview(null);
    try {
      await applyHotkey(value);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }

  async function pickPreset(hotkey: string) {
    if (recording) {
      finishingRef.current = true;
      setRecording(false);
      setPreview(null);
    }
    setError(null);
    try {
      await applyHotkey(hotkey);
      onChange(hotkey);
    } catch (e) {
      setError(`Не удалось назначить: ${e}`);
    }
  }

  const inPresets = presets.some((p) => p.value === value);

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-sm text-gray-600">Сейчас:</span>
        <kbd className="rounded-md bg-gray-100 px-2.5 py-1 font-mono text-sm font-medium text-gray-900 ring-1 ring-gray-200">
          {displayHotkey(value, os)}
        </kbd>
        {!inPresets && (
          <span className="text-xs text-gray-400">своя комбинация</span>
        )}
      </div>

      <div>
        <p className="mb-2 text-xs text-gray-500">
          Варианты для{" "}
          {os === "macos" ? "macOS" : os === "windows" ? "Windows" : "Linux"}
        </p>
        <div className="flex flex-wrap gap-2">
          {presets.map((p) => (
            <button
              key={p.value}
              type="button"
              onClick={() => pickPreset(p.value)}
              disabled={recording}
              className={`rounded-full px-3 py-1 text-sm transition-colors disabled:opacity-50 ${
                value === p.value
                  ? "bg-indigo-600 text-white"
                  : "bg-gray-100 text-gray-700 hover:bg-gray-200"
              }`}
            >
              {p.label}
            </button>
          ))}
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        {recording ? (
          <>
            <div
              ref={captureRef}
              tabIndex={0}
              role="status"
              className="rounded-lg border border-indigo-200 bg-indigo-50 px-3 py-2 text-sm text-indigo-800 outline-none ring-2 ring-indigo-300"
            >
              {preview
                ? `Отпустите: ${displayHotkey(preview, os)}`
                : "Жду сочетание клавиш… (Esc — отмена)"}
            </div>
            <button
              type="button"
              tabIndex={-1}
              onClick={cancelRecording}
              className="rounded-lg border border-gray-200 px-3 py-1.5 text-sm text-gray-700 hover:bg-gray-50"
            >
              Отмена
            </button>
          </>
        ) : (
          <button
            type="button"
            onClick={() => setRecording(true)}
            className="rounded-lg border border-gray-200 px-3 py-1.5 text-sm text-gray-700 hover:bg-gray-50"
          >
            Записать свою
          </button>
        )}
      </div>

      <p className="text-xs text-gray-400">
        Модификаторы + одна клавиша (например Ctrl+Space). Боковые кнопки мыши
        ОС не поддерживает. Активация сразу; «Сохранить» пишет в настройки.
      </p>
      {error && <p className="text-xs text-red-600">{error}</p>}
    </div>
  );
}

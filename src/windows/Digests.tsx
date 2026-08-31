import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";
import {
  DIGEST_KIND_LABELS,
  formatDate,
  generateDigestNow,
  listDigests,
  type Digest,
  type DigestKind,
} from "../lib/tauri";

export default function Digests() {
  const [digests, setDigests] = useState<Digest[]>([]);
  const [selected, setSelected] = useState<Digest | null>(null);
  const [filter, setFilter] = useState<string>("all");
  const [loading, setLoading] = useState(true);
  const [generating, setGenerating] = useState<DigestKind | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      try {
        let granted = await isPermissionGranted();
        if (!granted) {
          const result = await requestPermission();
          granted = result === "granted";
        }
      } catch {
        // optional
      }
    })();
  }, []);

  const load = useCallback(async () => {
    try {
      const data = await listDigests(filter === "all" ? undefined : filter);
      setDigests(data);
      setSelected((prev) => {
        if (!prev) return data[0] ?? null;
        return data.find((d) => d.id === prev.id) ?? data[0] ?? null;
      });
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [filter]);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    const unlisten = listen("digest-updated", () => {
      load();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [load]);

  async function handleGenerate(kind: DigestKind) {
    setGenerating(kind);
    setError(null);
    try {
      const digest = await generateDigestNow(kind);
      await load();
      setSelected(digest);
    } catch (e) {
      setError(String(e));
    } finally {
      setGenerating(null);
    }
  }

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-gray-200 bg-white px-6 py-3">
        <p className="text-xs text-gray-500">
          Планы дня / недели / месяца. Локальный TZ системы.
        </p>
        <div className="mt-2 flex flex-wrap gap-2">
          {(["daily", "weekly", "monthly"] as DigestKind[]).map((kind) => (
            <button
              key={kind}
              type="button"
              disabled={generating !== null}
              onClick={() => handleGenerate(kind)}
              className="rounded-lg border border-gray-200 bg-white px-3 py-1.5 text-xs text-gray-700 hover:bg-gray-50 disabled:opacity-50"
            >
              {generating === kind
                ? "Генерация…"
                : `Сгенерировать: ${DIGEST_KIND_LABELS[kind]}`}
            </button>
          ))}
        </div>
        {error && <p className="mt-2 text-xs text-red-600">{error}</p>}
      </div>

      <div className="flex gap-2 border-b border-gray-200 bg-white px-6 py-2">
        {["all", "daily", "weekly", "monthly"].map((f) => (
          <button
            key={f}
            type="button"
            onClick={() => setFilter(f)}
            className={`rounded-full px-3 py-1 text-xs ${
              filter === f
                ? "bg-indigo-600 text-white"
                : "bg-gray-100 text-gray-600 hover:bg-gray-200"
            }`}
          >
            {f === "all" ? "Все" : DIGEST_KIND_LABELS[f]}
          </button>
        ))}
      </div>

      <div className="flex min-h-0 flex-1">
        <aside className="w-72 overflow-y-auto border-r border-gray-200 bg-white">
          {loading ? (
            <p className="p-4 text-sm text-gray-400">Загрузка…</p>
          ) : digests.length === 0 ? (
            <p className="p-4 text-sm text-gray-400">Пока нет дайджестов</p>
          ) : (
            <ul className="divide-y divide-gray-100">
              {digests.map((d) => (
                <li key={d.id}>
                  <button
                    type="button"
                    onClick={() => setSelected(d)}
                    className={`w-full px-4 py-3 text-left hover:bg-gray-50 ${
                      selected?.id === d.id ? "bg-indigo-50" : ""
                    }`}
                  >
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-gray-900">
                        {DIGEST_KIND_LABELS[d.kind] ?? d.kind}
                      </span>
                      {d.source === "llm" && (
                        <span className="rounded bg-violet-100 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-violet-700">
                          AI
                        </span>
                      )}
                    </div>
                    <p className="mt-1 line-clamp-2 text-xs text-gray-500">{d.preview}</p>
                    <p className="mt-1 text-[10px] text-gray-400">{formatDate(d.createdAt)}</p>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </aside>

        <main className="flex-1 overflow-y-auto p-6">
          {!selected ? (
            <p className="text-sm text-gray-400">Выберите дайджест</p>
          ) : (
            <article>
              <div className="mb-4 flex items-center gap-2">
                <h2 className="text-base font-semibold text-gray-900">
                  {DIGEST_KIND_LABELS[selected.kind] ?? selected.kind}
                </h2>
                {selected.source === "llm" && (
                  <span className="rounded bg-violet-100 px-2 py-0.5 text-xs font-semibold uppercase text-violet-700">
                    AI
                  </span>
                )}
                <span className="text-xs text-gray-400">{selected.status}</span>
              </div>
              <p className="mb-4 text-xs text-gray-500">
                Период: {formatDate(selected.periodStart)} — {formatDate(selected.periodEnd)}
              </p>
              {selected.error && (
                <p className="mb-3 text-sm text-red-600">{selected.error}</p>
              )}
              <pre className="whitespace-pre-wrap rounded-lg border border-gray-200 bg-white p-4 text-sm text-gray-800">
                {selected.content || "—"}
              </pre>
            </article>
          )}
        </main>
      </div>
    </div>
  );
}

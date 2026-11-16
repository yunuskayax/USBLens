import { useEffect } from "react";
import type { Lang } from "../types";
import { dict } from "../i18n";
import { IconX } from "./Icons";

export type Toast = {
  id: number;
  kind: "info" | "ok" | "error" | "in" | "out";
  title: string;
  body?: string;
  action?: { label: string; onClick: () => void };
};

export function Toasts({ toasts, dismiss }: { toasts: Toast[]; dismiss: (id: number) => void }) {
  return (
    <div className="toasts" role="status">
      {toasts.map((t) => (
        <ToastItem key={t.id} toast={t} dismiss={dismiss} />
      ))}
    </div>
  );
}

function ToastItem({ toast, dismiss }: { toast: Toast; dismiss: (id: number) => void }) {
  useEffect(() => {
    const h = setTimeout(() => dismiss(toast.id), toast.action ? 15000 : toast.kind === "error" ? 7000 : 4500);
    return () => clearTimeout(h);
  }, [toast, dismiss]);
  return (
    <div className={`toast ${toast.kind}`}>
      <div className="grow">
        <b>{toast.title}</b>
        {toast.body && <div className="secondary">{toast.body}</div>}
        {toast.action && (
          <button className="btn primary toast-action" onClick={() => { toast.action!.onClick(); dismiss(toast.id); }}>
            {toast.action.label}
          </button>
        )}
      </div>
      <button className="icon-btn" onClick={() => dismiss(toast.id)} aria-label="close">
        <IconX size={14} />
      </button>
    </div>
  );
}

export interface ConfirmState {
  title: string;
  body: string;
  danger?: boolean;
  onConfirm: () => void;
}

export function ConfirmDialog({ lang, state, close }: { lang: Lang; state: ConfirmState; close: () => void }) {
  const t = dict[lang];
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && close();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [close]);
  return (
    <div className="modal-bg" onClick={close}>
      <div className="modal" onClick={(e) => e.stopPropagation()} role="dialog" aria-modal>
        <h3>{state.title}</h3>
        <p>{state.body}</p>
        <div className="modal-actions">
          <button className="btn" onClick={close} autoFocus>
            {t.cancel}
          </button>
          <button
            className={`btn ${state.danger ? "danger solid" : "primary"}`}
            onClick={() => {
              close();
              state.onConfirm();
            }}
          >
            {t.confirm}
          </button>
        </div>
      </div>
    </div>
  );
}

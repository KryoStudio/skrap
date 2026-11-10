// Toasts and the confirm dialog, shared by every page.

export interface Toast {
  id: number;
  kind: 'success' | 'error' | 'info';
  title: string;
  body?: string;
  action?: { label: string; run: () => void };
}

let next = 0;
export const toasts = $state<Toast[]>([]);

export function toast(t: Omit<Toast, 'id'>) {
  const id = next++;
  toasts.push({ ...t, id });
  if (toasts.length > 3) toasts.shift();
  setTimeout(() => dismissToast(id), t.action ? 7000 : 4500);
}

export function dismissToast(id: number) {
  const i = toasts.findIndex((t) => t.id === id);
  if (i >= 0) toasts.splice(i, 1);
}

interface ConfirmOptions {
  title: string;
  body: string;
  confirm: string;
}

export const dialog = $state({ open: false, options: null as ConfirmOptions | null, resolve: (_: boolean) => {} });

export function confirm(options: ConfirmOptions): Promise<boolean> {
  dialog.resolve(false);
  return new Promise((resolve) => Object.assign(dialog, { open: true, options, resolve }));
}

export function answer(yes: boolean) {
  dialog.open = false;
  dialog.resolve(yes);
  dialog.resolve = () => {};
}

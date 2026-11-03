import type { ReactNode } from "react";
import { createStore } from "zustand/vanilla";

export const dialog = createStore(() => ({
  open: false,
  title: "",
  content: null as ReactNode,
  wide: false,
}));

let back: HTMLElement | null = null;

export function openDialog(title: string, content: ReactNode, from?: HTMLElement | null, wide = false) {
  back = from ?? (document.activeElement as HTMLElement | null);
  dialog.setState({ open: true, title, content, wide });
}

export const closeDialog = () => dialog.setState({ open: false });

export const returnTo = (element: HTMLElement) => void (back = element);

export function closed() {
  dialog.setState({ open: false, content: null, wide: false });
  if (back?.isConnected) back.focus();
  back = null;
}

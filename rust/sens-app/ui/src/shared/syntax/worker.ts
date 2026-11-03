import { tokenize, type Painted } from "./tokenize";

self.addEventListener("message", async ({ data }: MessageEvent<{ id: number; text: string; language: string }>) => {
  let painted: Painted | null = null;
  try {
    painted = await tokenize(data.text, data.language);
  } catch {
    painted = null;
  }
  self.postMessage({ id: data.id, painted });
});

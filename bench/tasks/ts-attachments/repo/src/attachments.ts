import type { Attachment } from "./types.ts";

export function describe(attachment: Attachment): string {
  return attachment.name;
}

export function listing(attachments: Attachment[]): string {
  return attachments.map(describe).join("\n");
}

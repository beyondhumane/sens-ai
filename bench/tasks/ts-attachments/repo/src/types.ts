export interface Attachment {
  name: string;
  bytes: number;
  addedAt: Date;
}

export interface Quota {
  usedBytes: number;
  limitBytes: number;
  renewsAt: Date;
}

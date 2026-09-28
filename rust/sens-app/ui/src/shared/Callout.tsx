import type { ReactNode } from "react";
import { Icon } from "./Icon";
import "./callout.css";

export function Callout({ icon, title, said, role = "status", children }: { icon: string; title: string; said: string; role?: "alert" | "status"; children: ReactNode }) {
  return (
    <div className="callout" role={role}>
      <Icon svg={icon} />
      <div className="callout-text">
        <b>{title}</b>
        <span>{said}</span>
      </div>
      <div className="callout-actions">{children}</div>
    </div>
  );
}

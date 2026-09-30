import { useState, type ReactNode } from "react";

export function Switch({ id, on, save, children }: { id: string; on: boolean; save: (on: boolean) => Promise<unknown>; children: ReactNode }) {
  const [fault, setFault] = useState("");

  async function flip() {
    setFault("");
    try {
      await save(!on);
    } catch (reason) {
      setFault(String(reason));
    }
  }

  return (
    <>
      <div className="settings-switch">
        <button className="switch" id={id} role="switch" aria-checked={on} onClick={flip} />
        <label htmlFor={id}>{children}</label>
      </div>
      <p className="note fault" role="alert" hidden={!fault}>
        {fault}
      </p>
    </>
  );
}

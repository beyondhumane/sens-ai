// @vitest-environment jsdom
import { beforeEach, describe, expect, it } from "vitest";
import SCRIPT from "./page.js?raw";

interface Page {
  read: (interactive: boolean) => string;
  find: (query: string) => string;
  text: () => string;
  spot: (ref: string) => { x: number; y: number; name: string };
  focus: (ref: string, clear: boolean) => string;
  scroll: (ref: string | null, pages: number) => string;
}

const page = () => {
  new Function(SCRIPT)();
  return (window as unknown as { __sens: Page }).__sens;
};

beforeEach(() => {
  delete (window as unknown as { __sens?: Page }).__sens;
  document.title = "Tienda";
  document.body.innerHTML = `
    <nav aria-label="Principal"><a href="/carrito">Carrito <b>3</b></a></nav>
    <main>
      <h1>Zapatillas</h1>
      <label for="q">Buscar</label><input id="q" value="rojas">
      <input type="password" value="secreto" aria-label="Clave">
      <input type="checkbox" checked aria-label="Solo en stock">
      <button>Comprar</button>
      <div style="display:none"><button>Oculto</button></div>
      <p>Envío gratis desde 50 €.</p>
    </main>`;
  Element.prototype.scrollIntoView = () => {};
});

describe("what Claude reads of a page", () => {
  it("lists what can be used, each with a reference, and never a password", () => {
    const read = page().read(true);
    expect(read).toBe(
      [
        "Tienda",
        "http://localhost:3000/",
        "",
        'link "Carrito 3" [ref_1] href="/carrito"',
        'textbox "Buscar" [ref_2] value="rojas"',
        'textbox "Clave" [ref_3]',
        'checkbox "Solo en stock" [ref_4] checked',
        'button "Comprar" [ref_5]',
      ].join("\n"),
    );
  });

  it("keeps the same references and adds the structure when asked for all", () => {
    page().read(true);
    const all = page().read(false);
    expect(all).toContain('navigation "Principal" [ref_6]');
    expect(all).toContain('main "" [ref_7]');
    expect(all).toContain('  link "Carrito 3" [ref_1]');
    expect(all).toContain('  heading "Zapatillas"');
    expect(all).not.toContain("Oculto");
  });

  it("finds by name, role or placeholder", () => {
    expect(page().find("comprar")).toBe('button "Comprar" [ref_1]');
    expect(page().find("nada parecido")).toBe('Nothing on the page matches "nada parecido".');
  });

  it("reads the main text", () => {
    expect(page().text()).toContain("Envío gratis desde 50 €.");
  });

  it("aims at an element, focuses and clears a field, and says when a reference is stale", () => {
    const sens = page();
    sens.read(true);
    expect(sens.spot("ref_5").name).toBe("Comprar");
    expect(sens.focus("ref_2", true)).toBe("Buscar");
    expect(document.querySelector<HTMLInputElement>("#q")!.value).toBe("");
    expect(document.activeElement?.id).toBe("q");
    expect(() => sens.spot("ref_99")).toThrow("ref_99 is not on the page any more");
  });
});

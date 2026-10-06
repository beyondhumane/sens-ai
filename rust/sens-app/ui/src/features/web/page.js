window.__sens = window.__sens || (() => {
  const ROLES = {
    A: "link",
    BUTTON: "button",
    TEXTAREA: "textbox",
    SELECT: "combobox",
    IMG: "img",
    H1: "heading",
    H2: "heading",
    H3: "heading",
    H4: "heading",
    H5: "heading",
    H6: "heading",
    NAV: "navigation",
    MAIN: "main",
    HEADER: "banner",
    FOOTER: "contentinfo",
    FORM: "form",
    UL: "list",
    OL: "list",
    LI: "listitem",
    TABLE: "table",
    DIALOG: "dialog",
    SUMMARY: "button",
  };
  const INPUTS = { checkbox: "checkbox", radio: "radio", submit: "button", button: "button", reset: "button", range: "slider", search: "searchbox" };
  const ACTIVE = 'a[href],button,input:not([type=hidden]),textarea,select,summary,[role=button],[role=link],[role=checkbox],[role=radio],[role=tab],[role=menuitem],[role=option],[role=switch],[role=textbox],[contenteditable=""],[contenteditable=true],[onclick],[tabindex]:not([tabindex="-1"])';
  const SKIPPED = new Set(["SCRIPT", "STYLE", "NOSCRIPT", "TEMPLATE", "HEAD", "META", "LINK"]);
  const FIELDS = new Set(["INPUT", "TEXTAREA", "SELECT"]);
  const VALUELESS = new Set(["checkbox", "radio", "password", "submit", "button", "reset", "file"]);
  const WORDED = new Set(["", "link", "button", "heading", "tab", "menuitem", "option", "listitem", "summary", "checkbox", "radio", "switch", "img"]);
  const MOST = 50000;
  let made = 0;

  const squeeze = (text, most = 100) => String(text || "").replace(/\s+/g, " ").trim().slice(0, most);
  const roleOf = (el) => el.getAttribute("role") || (el.tagName === "INPUT" ? INPUTS[el.type] || "textbox" : ROLES[el.tagName] || "");
  const labelOf = (el) => (el.labels && el.labels[0] ? el.labels[0].innerText || el.labels[0].textContent : "");
  const nameOf = (el) =>
    squeeze(
      el.getAttribute("aria-label") ||
        el.getAttribute("alt") ||
        labelOf(el) ||
        el.getAttribute("placeholder") ||
        el.getAttribute("title") ||
        (FIELDS.has(el.tagName) || !WORDED.has(roleOf(el)) ? "" : el.innerText || el.textContent),
    );
  const hidden = (el) => {
    if (el.hidden || el.getAttribute("aria-hidden") === "true") return true;
    const style = getComputedStyle(el);
    return style.display === "none" || style.visibility === "hidden";
  };
  const refOf = (el) => {
    let ref = el.getAttribute("data-sens-ref");
    if (!ref) {
      ref = `ref_${++made}`;
      el.setAttribute("data-sens-ref", ref);
    }
    return ref;
  };
  const stateOf = (el) => {
    const parts = [];
    if (FIELDS.has(el.tagName) && !VALUELESS.has(el.type) && el.value) parts.push(`value="${squeeze(el.value, 80)}"`);
    if (el.checked) parts.push("checked");
    if (el.disabled) parts.push("disabled");
    if (el.tagName === "A" && el.getAttribute("href")) parts.push(`href="${squeeze(el.getAttribute("href"), 120)}"`);
    return parts.length ? ` ${parts.join(" ")}` : "";
  };
  const lineOf = (el, depth) => `${"  ".repeat(depth)}${roleOf(el) || el.tagName.toLowerCase()} "${nameOf(el)}" [${refOf(el)}]${stateOf(el)}`;
  const head = () => `${document.title}\n${location.href}\n`;

  const read = (interactive) => {
    const lines = [];
    const walk = (el, depth) => {
      if (SKIPPED.has(el.tagName) || hidden(el)) return;
      const active = el.matches(ACTIVE);
      const listed = active || (!interactive && roleOf(el));
      if (listed) lines.push(lineOf(el, depth));
      if (active && (el.tagName === "A" || el.tagName === "BUTTON")) return;
      for (const child of el.children) walk(child, listed ? depth + 1 : depth);
    };
    if (document.body) walk(document.body, 0);
    return `${head()}\n${lines.join("\n") || "(nothing on the page)"}`.slice(0, MOST);
  };

  const find = (query) => {
    const wanted = squeeze(query, 200).toLowerCase();
    const found = [];
    for (const el of document.querySelectorAll("body *")) {
      if (found.length >= 20) break;
      if (SKIPPED.has(el.tagName) || hidden(el)) continue;
      const own = [roleOf(el), nameOf(el), el.getAttribute("placeholder"), el.id].join(" ").toLowerCase();
      const leaf = el.matches(ACTIVE) || !el.children.length;
      if (leaf && own.includes(wanted)) found.push(lineOf(el, 0));
    }
    return found.length ? found.join("\n") : `Nothing on the page matches "${query}".`;
  };

  const text = () => {
    const root = document.querySelector("main, article") || document.body;
    const said = String((root && (root.innerText || root.textContent)) || "");
    return `${head()}\n${said.replace(/[ \t]+/g, " ").replace(/\n\s*\n+/g, "\n\n").trim().slice(0, MOST)}`;
  };

  const element = (ref) => {
    const el = document.querySelector(`[data-sens-ref="${ref}"]`);
    if (!el) throw new Error(`${ref} is not on the page any more; read the page again for fresh references.`);
    return el;
  };

  const spot = (ref) => {
    const el = element(ref);
    el.scrollIntoView({ block: "center", inline: "center" });
    const box = el.getBoundingClientRect();
    return { x: box.left + box.width / 2, y: box.top + box.height / 2, name: nameOf(el) };
  };

  const focus = (ref, clear) => {
    const el = element(ref);
    el.scrollIntoView({ block: "center" });
    el.focus();
    if (clear && "value" in el) {
      el.value = "";
      el.dispatchEvent(new Event("input", { bubbles: true }));
    }
    if (el.isContentEditable && clear) el.textContent = "";
    return nameOf(el);
  };

  const scroll = (ref, pages) => {
    if (ref) element(ref).scrollIntoView({ block: "center" });
    else window.scrollBy(0, pages * window.innerHeight * 0.8);
    return `Scrolled to ${Math.round(window.scrollY)} of ${Math.max(0, Math.round(document.documentElement.scrollHeight - window.innerHeight))} px.`;
  };

  return { read, find, text, spot, focus, scroll };
})();

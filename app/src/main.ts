import {
  type Api,
  type Hit,
  type Paste,
  type Summary,
  tauriApi,
} from "./api";

const el = {
  list: document.getElementById("list") as HTMLDivElement,
  preview: document.getElementById("preview") as HTMLElement,
  search: document.getElementById("search") as HTMLInputElement,
  status: document.getElementById("status") as HTMLElement,
  count: document.getElementById("count") as HTMLSpanElement,
};

type Row = {
  id: string;
  text: string;
  meta: string;
  enc: boolean;
  match?: { before: string; hit: string; after: string };
};

type Compose = {
  id: string | null;
  body: string;
  ttl: number;
  seal: boolean;
  label: string;
  pass: string;
  confirm: string;
};

let api: Api;
let items: Summary[] = [];
let hits: Hit[] = [];
let query = "";
let selectedId: string | null = null;
let paste: Paste | null = null;
let revealed: string | null = null;
let unlockPass: string | null = null;
let lockPass = "";
let lockError: string | null = null;
let mode: "browse" | "compose" = "browse";
let compose: Compose | null = null;
let confirmDelete: string | null = null;
let flashText: string | null = null;
let flashError = false;
let flashTimer: number | undefined;
let searchTimer: number | undefined;
let focusTarget: "editor" | "lock" | null = null;

void boot();

async function boot(): Promise<void> {
  api = await pickApi();
  bind();
  await refresh();
}

async function pickApi(): Promise<Api> {
  if ("__TAURI_INTERNALS__" in window) {
    return tauriApi;
  }
  if (import.meta.env.DEV) {
    const { mockApi } = await import("./mock");
    return mockApi;
  }
  throw new Error("no tauri ipc");
}

function bind(): void {
  document.addEventListener("keydown", onKey);
  el.search.addEventListener("input", () => {
    window.clearTimeout(searchTimer);
    searchTimer = window.setTimeout(() => void setQuery(el.search.value), 100);
  });
  el.list.addEventListener("click", (event) => {
    const row = (event.target as HTMLElement).closest<HTMLButtonElement>(".row");
    if (!row) return;
    const index = Number(row.dataset.index);
    void select(index);
  });
}

async function refresh(keep?: string): Promise<void> {
  const all = await api.list();
  items = all;
  if (keep) {
    selectedId = keep;
  }
  if (!rows().some((r) => r.id === selectedId)) {
    selectedId = rows()[0]?.id ?? null;
  }
  await loadPaste();
  render();
}

function rows(): Row[] {
  if (!query) {
    return items.map((item) => ({
      id: item.id,
      text: item.name,
      meta: ago(item.createdAt),
      enc: item.enc,
    }));
  }
  return hits.map((hit) => ({
    id: hit.id,
    text: hit.name,
    meta: "",
    enc: false,
    match: splitSnippet(hit.snippet) ?? undefined,
  }));
}

function splitSnippet(snippet: string): Row["match"] | null {
  const open = snippet.indexOf("\u0001");
  const close = snippet.indexOf("\u0002");
  if (open < 0 || close < 0) return null;
  return {
    before: snippet.slice(Math.max(0, open - 28), open),
    hit: snippet.slice(open + 1, close),
    after: snippet.slice(close + 1, close + 61),
  };
}

async function select(index: number): Promise<void> {
  const row = rows()[index];
  if (!row) return;
  selectedId = row.id;
  mode = "browse";
  compose = null;
  confirmDelete = null;
  await loadPaste();
  render();
}

async function loadPaste(): Promise<void> {
  if (!selectedId || query) {
    if (!selectedId) paste = null;
  }
  if (!selectedId) {
    paste = null;
    return;
  }
  try {
    paste = await api.get(selectedId);
    revealed = null;
    unlockPass = null;
    lockPass = "";
    lockError = null;
  } catch (error) {
    fail(error);
    paste = null;
  }
}

async function setQuery(value: string): Promise<void> {
  query = value.trim();
  if (!query) {
    hits = [];
    if (!selectedId) selectedId = items[0]?.id ?? null;
    await loadPaste();
    render();
    return;
  }
  const found = await api.search(query, 20);
  if (query !== value.trim()) return;
  hits = found;
  if (!hits.some((h) => h.id === selectedId)) {
    selectedId = hits[0]?.id ?? null;
  }
  await loadPaste();
  render();
}

function render(): void {
  renderList();
  renderPreview();
  renderStatus();
  el.count.textContent = query
    ? `${hits.length} hit${hits.length === 1 ? "" : "s"}`
    : `${items.length} paste${items.length === 1 ? "" : "s"}`;

  if (focusTarget === "editor") {
    const editor = el.preview.querySelector<HTMLTextAreaElement>(".editor");
    editor?.focus();
  }
  if (focusTarget === "lock") {
    const input = el.preview.querySelector<HTMLInputElement>(".lock input");
    input?.focus();
  }
  focusTarget = null;
}

function renderList(): void {
  const list = rows();
  if (!list.length) {
    const empty = node("div", "list-empty");
    empty.textContent = query ? "no matches" : "nothing stored yet";
    el.list.replaceChildren(empty);
    return;
  }
  const nodes = list.map((row, index) => {
    const button = node("button", "row");
    button.type = "button";
    button.dataset.index = String(index);
    button.setAttribute("role", "option");
    if (row.id === selectedId) button.setAttribute("aria-selected", "true");

    button.append(node("span", "pid", row.id));

    const label = node("span", "label");
    if (row.match) {
      label.append(document.createTextNode("…"));
      label.append(document.createTextNode(row.match.before.trimStart()));
      label.append(node("mark", "", row.match.hit));
      label.append(document.createTextNode(row.match.after));
    } else {
      label.textContent = row.text;
      if (row.enc) label.prepend(node("span", "seal", "▪ "));
    }
    button.append(label, node("span", "meta", row.meta));
    return button;
  });
  el.list.replaceChildren(...nodes);
}

function renderPreview(): void {
  if (mode === "compose" && compose) {
    renderCompose(compose);
    return;
  }
  if (!paste) {
    const empty = node("div", "empty");
    empty.append(node("p", "empty-title", "nothing selected"));
    empty.append(node("p", "empty-hint", "N writes a paste · / searches"));
    el.preview.replaceChildren(empty);
    return;
  }

  const item = items.find((i) => i.id === paste!.id);
  const head = node("div", "phead");
  head.append(node("span", "pid", paste.id));
  const meta = [
    item ? fmtSize(item.size) : "",
    item ? ago(item.createdAt) : "",
    paste.enc ? (revealed === null ? "sealed" : "sealed · unlocked") : "",
  ]
    .filter(Boolean)
    .join("  ·  ");
  head.append(node("span", "pmeta", meta));
  el.preview.replaceChildren(head);

  if (paste.enc && revealed === null) {
    renderLock();
    return;
  }

  const body = paste.enc ? (revealed ?? "") : paste.body;
  const pre = node("pre", "body");
  pre.textContent = body;
  el.preview.append(pre);
}

function renderLock(): void {
  const wrap = node("div", "lock");
  wrap.append(node("p", "lock-title", "sealed paste"));
  wrap.append(
    node(
      "p",
      "lock-hint",
      "The text is encrypted in the store. The passphrase is never saved.",
    ),
  );

  const input = node("input", "lock-input");
  input.type = "password";
  input.placeholder = "passphrase";
  input.value = lockPass;
  input.addEventListener("input", () => {
    lockPass = input.value;
  });
  input.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      void unlock();
    }
  });

  const button = node("button", "lock-button", "Unlock");
  button.type = "button";
  button.addEventListener("click", () => void unlock());

  const row = node("div", "lock-row");
  row.append(input, button);
  wrap.append(row);

  if (lockError) {
    wrap.append(node("p", "lock-error", lockError));
  }
  el.preview.append(wrap);
}

function renderCompose(draft: Compose): void {
  const head = node("div", "phead");
  head.append(node("span", "pid", draft.id ? `edit ${draft.id}` : "new paste"));

  const save = node("button", "save", "save");
  save.type = "button";
  save.addEventListener("click", () => void saveCompose());
  head.append(save);

  const editor = node("textarea", "editor");
  editor.spellcheck = false;
  editor.placeholder = "type or paste text…";
  editor.value = draft.body;

  const count = node("span", "charcount");
  const updateCount = () => {
    const n = editor.value.length;
    count.textContent = `${n} char${n === 1 ? "" : "s"}`;
  };
  updateCount();
  editor.addEventListener("input", () => {
    draft.body = editor.value;
    updateCount();
  });

  const options = node("div", "options");

  const ttlLabel = node("label", "opt");
  ttlLabel.append(node("span", "opt-label", "expires"));
  const ttl = node("select", "opt-select");
  for (const [value, text] of [
    ["0", "never"],
    ["3600", "in 1 hour"],
    ["86400", "in 1 day"],
    ["604800", "in 7 days"],
  ] as const) {
    const option = node("option", "", text);
    option.value = value;
    ttl.append(option);
  }
  ttl.value = String(draft.ttl);
  ttl.addEventListener("change", () => {
    draft.ttl = Number(ttl.value);
  });
  ttlLabel.append(ttl);

  const sealLabel = node("label", "opt");
  const seal = node("input", "opt-check");
  seal.type = "checkbox";
  seal.checked = draft.seal;
  seal.addEventListener("change", () => {
    draft.seal = seal.checked;
    focusTarget = "editor";
    render();
  });
  sealLabel.append(seal, node("span", "opt-label", "seal"));

  options.append(ttlLabel, sealLabel, count);

  if (draft.seal) {
    const secrets = node("div", "secrets");

    const label = node("input", "opt-text");
    label.placeholder = "label (stays readable)";
    label.value = draft.label;
    label.addEventListener("input", () => {
      draft.label = label.value;
    });

    const pass = node("input", "opt-text");
    pass.type = "password";
    pass.placeholder = "passphrase";
    pass.value = draft.pass;
    pass.addEventListener("input", () => {
      draft.pass = pass.value;
    });

    const again = node("input", "opt-text");
    again.type = "password";
    again.placeholder = "passphrase again";
    again.value = draft.confirm;
    again.addEventListener("input", () => {
      draft.confirm = again.value;
    });

    secrets.append(label, pass, again);
    el.preview.replaceChildren(head, editor, options, secrets);
    return;
  }

  el.preview.replaceChildren(head, editor, options);
}

function renderStatus(): void {
  const left = node("div", "hints");
  const right = node("div", "flash");

  if (confirmDelete) {
    left.append(node("span", "danger", `delete ${confirmDelete}?`));
    left.append(hint("Enter", "confirm"), hint("Esc", "cancel"));
  } else if (mode === "compose") {
    left.append(hint("Ctrl+Enter", "save"), hint("Esc", "cancel"));
    right.append(node("span", "muted", "one paste, 64 KB at most"));
  } else if (paste?.enc && revealed === null) {
    left.append(hint("type", "passphrase"), hint("Enter", "unlock"));
  } else {
    left.append(
      hint("N", "new"),
      hint("E", "edit"),
      hint("⌫", "delete"),
      hint("Ctrl+E", "export"),
    );
    right.append(hint("↑↓", "move"), hint("Enter", "copy"));
  }

  if (flashText) {
    right.replaceChildren(node("span", flashError ? "danger" : "ok", flashText));
  }

  el.status.replaceChildren(left, right);
}

function hint(key: string, label: string): HTMLElement {
  const wrap = node("span", "hint");
  wrap.append(node("kbd", "", key), document.createTextNode(" " + label));
  return wrap;
}

function node<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  cls = "",
  text?: string,
): HTMLElementTagNameMap[K] {
  const element = document.createElement(tag);
  if (cls) element.className = cls;
  if (text !== undefined) element.textContent = text;
  return element;
}

async function onKey(event: KeyboardEvent): Promise<void> {
  const target = event.target as HTMLElement;
  const typing =
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement;

  if (event.key === "Escape") {
    if (mode === "compose") {
      mode = "browse";
      compose = null;
      render();
      return;
    }
    if (confirmDelete) {
      confirmDelete = null;
      render();
      return;
    }
    if (target === el.search && el.search.value) {
      el.search.value = "";
      void setQuery("");
      return;
    }
    if (typing) target.blur();
    return;
  }

  if (mode === "compose" && (event.ctrlKey || event.metaKey) && event.key === "Enter") {
    event.preventDefault();
    await saveCompose();
    return;
  }

  if (typing) {
    return;
  }

  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "e") {
    event.preventDefault();
    await exportSelected();
    return;
  }

  if (confirmDelete) {
    if (event.key === "Enter") {
      event.preventDefault();
      await removeConfirmed(confirmDelete);
    }
    return;
  }

  const list = rows();
  const index = list.findIndex((r) => r.id === selectedId);

  switch (event.key) {
    case "/":
      event.preventDefault();
      el.search.focus();
      el.search.select();
      return;
    case "j":
    case "ArrowDown":
      event.preventDefault();
      if (list.length) await select(Math.min(list.length - 1, index + 1));
      return;
    case "k":
    case "ArrowUp":
      event.preventDefault();
      if (list.length) await select(Math.max(0, index - 1));
      return;
    case "n":
      event.preventDefault();
      openCompose();
      return;
    case "e":
      event.preventDefault();
      if (paste) openCompose(paste);
      return;
    case "Enter":
      event.preventDefault();
      await copySelected();
      return;
    case "Delete":
    case "Backspace":
      if (selectedId) {
        confirmDelete = selectedId;
        render();
      }
      return;
  }
}

function openCompose(pasteToEdit?: Paste): void {
  if (pasteToEdit?.enc) {
    flash(`${pasteToEdit.id} is sealed, edit needs its passphrase`, true);
    return;
  }
  mode = "compose";
  confirmDelete = null;
  compose = {
    id: pasteToEdit?.id ?? null,
    body: pasteToEdit?.body ?? "",
    ttl: 0,
    seal: false,
    label: "",
    pass: "",
    confirm: "",
  };
  focusTarget = "editor";
  render();
}

async function saveCompose(): Promise<void> {
  if (!compose) return;
  const draft = compose;
  if (!draft.body.trim()) {
    flash("nothing to store", true);
    return;
  }
  if (draft.seal) {
    if (!draft.pass) {
      flash("passphrase required", true);
      return;
    }
    if (draft.pass !== draft.confirm) {
      flash("passphrases differ", true);
      return;
    }
  }
  try {
    if (draft.id) {
      await api.update(draft.id, draft.body);
      mode = "browse";
      compose = null;
      await refresh(draft.id);
      flash(`updated ${draft.id}`);
    } else {
      const id = await api.add({
        body: draft.body,
        label: draft.seal ? draft.label.trim() || null : null,
        secret: draft.seal ? draft.pass : null,
        ttlSeconds: draft.ttl > 0 ? draft.ttl : null,
      });
      mode = "browse";
      compose = null;
      await refresh(id);
      flash(`saved ${id}`);
    }
  } catch (error) {
    fail(error);
    render();
  }
}

async function copySelected(): Promise<void> {
  if (!paste) return;
  if (paste.enc && revealed === null) {
    focusTarget = "lock";
    render();
    return;
  }
  try {
    await api.copy(paste.id, unlockPass);
    flash(`copied ${paste.id}`);
  } catch (error) {
    fail(error);
  }
}

async function exportSelected(): Promise<void> {
  if (!paste) return;
  if (paste.enc && revealed === null) {
    flash("unlock first", true);
    return;
  }
  try {
    const path = await api.export(paste.id, unlockPass);
    if (path) {
      flash(`exported ${path}`);
    }
  } catch (error) {
    fail(error);
  }
}

async function unlock(): Promise<void> {
  if (!paste) return;
  const id = paste.id;
  try {
    revealed = await api.reveal(id, lockPass);
    unlockPass = lockPass;
    lockError = null;
    render();
    flash(`unlocked ${id}`);
  } catch (error) {
    lockError = String(error);
    render();
    focusTarget = "lock";
  }
}

async function removeConfirmed(id: string): Promise<void> {
  try {
    await api.remove(id);
    confirmDelete = null;
    await refresh();
    flash(`deleted ${id}`);
  } catch (error) {
    fail(error);
  }
}

function flash(text: string, error = false): void {
  flashText = text;
  flashError = error;
  renderStatus();
  window.clearTimeout(flashTimer);
  flashTimer = window.setTimeout(() => {
    flashText = null;
    renderStatus();
  }, 2400);
}

function fail(error: unknown): void {
  flash(String(error), true);
}

function ago(createdAt: number): string {
  const seconds = Math.max(0, Math.floor((Date.now() - createdAt) / 1000));
  if (seconds < 60) return "now";
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
  if (seconds < 86_400) return `${Math.floor(seconds / 3600)}h`;
  return `${Math.floor(seconds / 86_400)}d`;
}

function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

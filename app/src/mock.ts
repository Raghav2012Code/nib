// Development-only stand-in for the Tauri IPC, so the window can be designed
// and clicked in a plain browser. Never imported in a production build.
import type { Api, Hit, Paste, Summary } from "./api";

type Row = { paste: Paste; createdAt: number; secret: string | null };

const seed = (): Row[] => {
  const now = Date.now();
  const make = (
    id: string,
    body: string,
    minutes: number,
    secret: string | null = null,
  ): Row => ({
    paste: {
      id,
      body,
      enc: secret !== null,
      label: secret ? "deploy key" : null,
    },
    createdAt: now - minutes * 60_000,
    secret,
  });
  return [
    make(
      "k3x9",
      `fn parse_args() -> Args {
    let mut args = std::env::args().skip(1);
    Args {
        first: args.next(),
        rest: args.collect(),
    }
}`,
      2,
    ),
    make(
      "m7qa",
      "kubectl get pods -A --field-selector status.phase!=Running",
      47,
    ),
    make(
      "p2wn",
      "SELECT id, body FROM pastes WHERE expires_at IS NULL ORDER BY created_at DESC;",
      180,
    ),
    make("r8ce", "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAI...", 1_440, "pw"),
    make(
      "t4mv",
      `cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test`,
      2_880,
    ),
  ];
};

let rows = seed();
let counter = 0;

const name = (paste: Paste): string => {
  if (paste.enc) {
    return paste.label ?? `secret ${paste.id}`;
  }
  const line = paste.body.split("\n").find((l) => l.trim().length > 0) ?? "";
  const clean = line.trim().replace(/\s+/g, " ");
  return clean.length > 80 ? `${clean.slice(0, 80)}…` : clean || "untitled";
};

const summary = (row: Row): Summary => ({
  id: row.paste.id,
  name: name(row.paste),
  size: new TextEncoder().encode(row.paste.body).length,
  createdAt: row.createdAt,
  enc: row.paste.enc,
});

const find = (id: string): Row => {
  const row =
    rows.find((r) => r.paste.id === id) ??
    rows.find((r) => r.paste.id.startsWith(id));
  if (!row) {
    throw `no paste ${id}`;
  }
  return row;
};

export const mockApi: Api = {
  async list() {
    return rows
      .slice()
      .sort((a, b) => b.createdAt - a.createdAt)
      .map(summary);
  },

  async get(id) {
    return { ...find(id).paste };
  },

  async reveal(id, passphrase) {
    const row = find(id);
    if (row.secret !== passphrase) {
      throw `cannot unlock ${row.paste.id}`;
    }
    return row.paste.body;
  },

  async search(query) {
    const q = query.toLowerCase();
    const hits: Hit[] = [];
    for (const row of rows) {
      const text = row.paste.enc ? (row.paste.label ?? "") : row.paste.body;
      const at = text.toLowerCase().indexOf(q);
      if (at < 0) {
        continue;
      }
      const before = text.slice(Math.max(0, at - 12), at);
      const match = text.slice(at, at + q.length);
      const after = text.slice(at + q.length, at + q.length + 24);
      hits.push({
        id: row.paste.id,
        name: name(row.paste),
        snippet: `${before}\u0001${match}\u0002${after}`,
      });
    }
    return hits;
  },

  async add(input) {
    counter += 1;
    const id = `n${counter}${"xyz"[counter % 3]}`;
    rows.unshift({
      paste: {
        id,
        body: input.body,
        enc: input.secret !== null,
        label: input.label,
      },
      createdAt: Date.now(),
      secret: input.secret,
    });
    return id;
  },

  async update(id, body) {
    const row = find(id);
    row.paste.body = body;
    row.createdAt = Date.now();
  },

  async remove(id) {
    const row = find(id);
    rows = rows.filter((r) => r !== row);
  },

  async copy(id, passphrase) {
    const row = find(id);
    if (row.secret !== null && row.secret !== passphrase) {
      throw `cannot unlock ${row.paste.id}`;
    }
    await navigator.clipboard?.writeText(row.paste.body);
  },

  async export(id) {
    return `nib-${id}.txt`;
  },
};

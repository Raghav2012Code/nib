import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

export type Summary = {
  id: string;
  name: string;
  size: number;
  createdAt: number;
  enc: boolean;
};

export type Hit = {
  id: string;
  name: string;
  snippet: string;
};

export type Paste = {
  id: string;
  body: string;
  enc: boolean;
  label: string | null;
};

export type AddInput = {
  body: string;
  label: string | null;
  secret: string | null;
  ttlSeconds: number | null;
};

export interface Api {
  list(limit?: number): Promise<Summary[]>;
  get(id: string): Promise<Paste>;
  reveal(id: string, passphrase: string): Promise<string>;
  search(query: string, limit?: number): Promise<Hit[]>;
  add(input: AddInput): Promise<string>;
  update(id: string, body: string): Promise<void>;
  remove(id: string): Promise<void>;
  copy(id: string, passphrase: string | null): Promise<void>;
  export(id: string, passphrase: string | null): Promise<string | null>;
}

export const tauriApi: Api = {
  list: (limit = 500) => invoke<Summary[]>("list_pastes", { limit }),
  get: (id) => invoke<Paste>("get_paste", { id }),
  reveal: (id, passphrase) =>
    invoke<string>("reveal_paste", { id, passphrase }),
  search: (query, limit = 20) =>
    invoke<Hit[]>("search_pastes", { query, limit }),
  add: (input) =>
    invoke<string>("add_paste", {
      body: input.body,
      label: input.label,
      secret: input.secret,
      ttlSeconds: input.ttlSeconds,
    }),
  update: (id, body) => invoke<void>("update_paste", { id, body }),
  remove: (id) => invoke<void>("delete_paste", { id }),
  copy: (id, passphrase) => invoke<void>("copy_paste", { id, passphrase }),
  export: async (id, passphrase) => {
    const path = await save({
      defaultPath: `nib-${id}.txt`,
      filters: [{ name: "Text", extensions: ["txt"] }],
    });
    if (!path) {
      return null;
    }
    await invoke<void>("export_paste", { id, path, passphrase });
    return path;
  },
};

// WASM bridge — lazy-loads the rune-forge WASM module and wraps exports.
// All components share a single WASM instance via this module.

interface RuneForgeWasm {
  compile_policy(source: string, format: string): string;
  parse_policy(source: string): string;
  decode_rune(raw: string): string;
  generate_policy_from_spec(spec: string): string;
  create_rune(secret_hex: string, restrictions_raw: string): string;
  decode_rune_base64(rune_base64: string): string;
  verify_rune(secret_hex: string, rune_base64: string): boolean;
}

type WasmState =
  | { status: "idle" }
  | { status: "loading"; promise: Promise<void> }
  | { status: "ready"; wasm: RuneForgeWasm }
  | { status: "error"; error: string };

let state: WasmState = { status: "idle" };

async function loadWasm(): Promise<void> {
  const wasmUrl = new URL("rune_forge_wasm_bg.wasm", import.meta.url).href;
  const glueUrl = new URL("rune_forge_wasm.js", import.meta.url).href;
  const glue = await import(/* @vite-ignore */ glueUrl);
  await glue.default(wasmUrl);
  state = {
    status: "ready",
    wasm: {
      compile_policy: glue.compile_policy,
      parse_policy: glue.parse_policy,
      decode_rune: glue.decode_rune,
      generate_policy_from_spec: glue.generate_policy_from_spec,
      create_rune: glue.create_rune,
      decode_rune_base64: glue.decode_rune_base64,
      verify_rune: glue.verify_rune,
    },
  };
}

export async function ensureWasm(): Promise<RuneForgeWasm> {
  if (state.status === "ready") return state.wasm;
  if (state.status === "error") throw new Error(state.error);
  if (state.status === "loading") {
    await state.promise;
    if (state.status === "ready") return state.wasm;
    throw new Error("WASM failed to load");
  }
  const promise = loadWasm().catch((e) => {
    state = { status: "error", error: String(e) };
    throw e;
  });
  state = { status: "loading", promise };
  await promise;
  if (state.status === "ready") return state.wasm;
  throw new Error("WASM failed to load");
}

export function compilePolicy(source: string, format: string): Promise<string> {
  return ensureWasm().then((w) => w.compile_policy(source, format));
}

export function parsePolicy(source: string): Promise<string> {
  return ensureWasm().then((w) => w.parse_policy(source));
}

export function decodeRune(raw: string): Promise<string> {
  return ensureWasm().then((w) => w.decode_rune(raw));
}

export function generatePolicy(spec: string): Promise<string> {
  return ensureWasm().then((w) => w.generate_policy_from_spec(spec));
}

export function createRune(secretHex: string, restrictionsRaw: string): Promise<string> {
  return ensureWasm().then((w) => w.create_rune(secretHex, restrictionsRaw));
}

export function decodeRuneBase64(runeBase64: string): Promise<string> {
  return ensureWasm().then((w) => w.decode_rune_base64(runeBase64));
}

export function verifyRune(secretHex: string, runeBase64: string): Promise<boolean> {
  return ensureWasm().then((w) => w.verify_rune(secretHex, runeBase64));
}

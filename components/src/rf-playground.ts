import { LitElement, html, css } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { compilePolicy, createRune } from "./wasm-bridge.js";
import "./rf-output.js";

const EXAMPLES: Record<string, string> = {
  "simple.rf": `allow methods: listfunds, listpeerchannels, getinfo`,
  "readonly.rf": `allow methods: listfunds, listpeerchannels, listchannels, listpays, listinvoices, getinfo, waitanyinvoice`,
  "tagged.rf": `tag: operator_id default-operator

allow methods: listfunds, listpeerchannels, fundchannel, close, invoice, xpay

when fundchannel:
  pnameamount < 1000001

when xpay:
  pnameamount_msat < 1000000001 or pnameamount_msat !`,
  "advanced.rf": `id: 02abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab

tag: operator_id advanced-operator
tag: version 2

allow methods: listfunds, listpeerchannels, listchannels, listpays, listinvoices, getinfo, fundchannel, close, invoice, xpay, waitanyinvoice

when fundchannel:
  pnameamount < 1000001

when xpay:
  (pnameamount_msat < 1000000001 or pnameamount_msat !) and rate = 10

when close:
  pnamedestination = bc1qexamplecoldwalletaddress

global:
  per = 1min`,
};

@customElement("rf-playground")
export class RfPlayground extends LitElement {
  @property() source = EXAMPLES["simple.rf"];
  @property() format: "json" | "cln" | "raw" | "rune" = "json";
  @property({ type: Boolean }) readonly = false;

  @state() private _output = "";
  @state() private _error = "";
  @state() private _status = "";
  @state() private _runeOutput = "";
  @state() private _secret = "0000000000000000000000000000000000000000000000000000000000000000";
  @state() private _debounceTimer: ReturnType<typeof setTimeout> | null = null;
  @state() private _shareMsg = "";

  static styles = css`
    :host { display: block; border: 1px solid #e2e4e8; border-radius: 8px; overflow: hidden; background: #fff; }
    .toolbar { display: flex; justify-content: space-between; align-items: center; padding: 0.4rem 1rem; border-bottom: 1px solid #e2e4e8; font-size: 0.75rem; background: #f7f8fa; }
    .toolbar select { background: #fff; border: 1px solid #e2e4e8; border-radius: 4px; color: #0c0c0f; padding: 0.2rem 0.4rem; font-size: 0.75rem; }
    .toolbar button { background: #00c3ff; border: none; color: #fff; border-radius: 4px; padding: 0.25rem 0.7rem; font-size: 0.75rem; cursor: pointer; font-weight: 600; }
    .toolbar button:hover { background: #0088b3; }
    .share-wrap { position: relative; }
    .share-toast { position: absolute; top: 110%; right: 0; background: #333; color: #fff; padding: 0.3rem 0.6rem; border-radius: 4px; font-size: 0.65rem; white-space: nowrap; pointer-events: none; opacity: 0; transition: opacity 0.15s; }
    .share-toast.visible { opacity: 1; }
    .editor-split { display: flex; min-height: 200px; }
    .pane { flex: 1; display: flex; flex-direction: column; }
    .pane + .pane { border-left: 1px solid #e2e4e8; }
    .pane-header { padding: 0.3rem 0.8rem; font-size: 0.65rem; color: #999; border-bottom: 1px solid #ebedef; text-transform: uppercase; letter-spacing: 0.05em; background: #f7f8fa; }
    textarea { flex: 1; border: none; padding: 0.8rem; font-family: "JetBrains Mono", "Fira Code", monospace; font-size: 0.8rem; line-height: 1.8; resize: none; background: #f7f8fa; color: #0c0c0f; outline: none; tab-size: 2; }
    .status { padding: 0.3rem 1rem; border-top: 1px solid #e2e4e8; font-size: 0.65rem; display: flex; justify-content: space-between; background: #f7f8fa; }
    .status-ok { color: #16a34a; }
    .status-err { color: #dc2626; }
    .status-version { color: #999; }
  `;

  connectedCallback() {
    super.connectedCallback();
    this._loadFromHash();
    window.addEventListener("hashchange", () => this._loadFromHash());
    this._compile();
  }

  private _loadFromHash() {
    const hash = window.location.hash;
    if (hash.startsWith("#policy=")) {
      try { this.source = decodeURIComponent(hash.slice(8)); } catch { /* ignore bad hash */ }
    }
  }

  render() {
    return html`
      <div class="toolbar">
        <div style="display:flex;gap:0.6rem;align-items:center">
          <span style="color:#666">Examples:</span>
          <select @change=${this._loadExample}>
            ${Object.keys(EXAMPLES).map(name => html`<option value=${name}>${name}</option>`)}
          </select>
        </div>
        <div class="share-wrap">
          <button @click=${this._share} title="Copy a shareable URL with the current policy">Share</button>
          <span class="share-toast ${this._shareMsg ? "visible" : ""}">${this._shareMsg}</span>
        </div>
      </div>
      <div class="editor-split">
        <div class="pane">
          <div class="pane-header">Policy</div>
          <textarea .value=${this.source} @input=${this._onInput} ?readonly=${this.readonly} spellcheck="false"></textarea>
        </div>
        <div class="pane">
          <rf-output .output=${this._output} .format=${this.format} .error=${this._error} .runeOutput=${this._runeOutput} .secret=${this._secret} @format-change=${this._onFormatChange} @secret-change=${this._onSecretChange}></rf-output>
        </div>
      </div>
      <div class="status">
        <span class=${this._error ? "status-err" : "status-ok"}>${this._status}</span>
        <span class="status-version">rune-reactor v0.1.0 (WASM)</span>
      </div>
    `;
  }

  private _onInput(e: InputEvent) {
    this.source = (e.target as HTMLTextAreaElement).value;
    if (this._debounceTimer) clearTimeout(this._debounceTimer);
    this._debounceTimer = setTimeout(() => this._compile(), 150);
  }

  private _onFormatChange(e: CustomEvent) {
    this.format = e.detail;
    this._compile();
  }

  private _onSecretChange(e: CustomEvent) {
    this._secret = e.detail;
    this._compile();
  }

  private _loadExample(e: Event) {
    const name = (e.target as HTMLSelectElement).value;
    if (EXAMPLES[name]) { this.source = EXAMPLES[name]; this._compile(); }
  }

  private async _compile() {
    try {
      const fmt = this.format === "rune" ? "raw" : this.format;
      this._output = await compilePolicy(this.source, fmt);
      this._error = "";
      if (this.format === "rune") {
        try {
          this._runeOutput = await createRune(this._secret, this._output);
        } catch (re) {
          this._runeOutput = "";
          this._error = String(re);
        }
      }
      try {
        const jsonOut = await compilePolicy(this.source, "json");
        const parsed = JSON.parse(jsonOut);
        this._status = `✓ Compiled — ${parsed.length} restriction${parsed.length !== 1 ? "s" : ""}`;
      } catch { this._status = "✓ Compiled"; }
    } catch (e) {
      this._error = String(e);
      this._output = "";
      this._runeOutput = "";
      this._status = `✗ ${this._error}`;
    }
  }

  private async _share() {
    const hash = `#policy=${encodeURIComponent(this.source)}`;
    window.location.hash = hash;
    await navigator.clipboard.writeText(window.location.href);
    this._shareMsg = "Shareable URL copied to clipboard";
    setTimeout(() => { this._shareMsg = ""; }, 2000);
  }
}

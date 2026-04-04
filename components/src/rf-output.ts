import { LitElement, html, css, nothing } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("rf-output")
export class RfOutput extends LitElement {
  @property() output = "";
  @property() format: "json" | "cln" | "raw" | "rune" = "json";
  @property() error = "";
  @property() runeOutput = "";
  @property() secret = "0000000000000000000000000000000000000000000000000000000000000000";

  static styles = css`
    :host { display: block; font-family: "JetBrains Mono", "Fira Code", monospace; font-size: 0.85rem; }
    .toolbar { display: flex; gap: 0.8rem; padding: 0.4rem 0.8rem; border-bottom: 1px solid #e2e4e8; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; background: #f7f8fa; }
    .toolbar button { background: none; border: none; cursor: pointer; padding: 0.2rem 0; color: #999; font-size: inherit; text-transform: inherit; letter-spacing: inherit; font-family: inherit; }
    .toolbar button.active { color: #0088b3; font-weight: 600; }
    .toolbar button:hover { color: #0088b3; }
    .copy-btn { margin-left: auto; cursor: pointer; color: #999; background: none; border: none; font-size: inherit; font-family: inherit; }
    .copy-btn:hover { color: #0088b3; }
    .output { padding: 0.8rem; white-space: pre-wrap; word-break: break-all; min-height: 3rem; color: #0c0c0f; }
    .error { color: #dc2626; padding: 0.8rem; white-space: pre-wrap; }
    .secret-row { display: flex; gap: 0.5rem; align-items: center; padding: 0.4rem 0.8rem; border-bottom: 1px solid #e2e4e8; background: #f7f8fa; font-size: 0.75rem; }
    .secret-row label { color: #666; white-space: nowrap; font-family: system-ui, -apple-system, sans-serif; text-transform: none; letter-spacing: normal; }
    .secret-row input { flex: 1; border: 1px solid #e2e4e8; border-radius: 4px; padding: 0.25rem 0.4rem; font-family: "JetBrains Mono", "Fira Code", monospace; font-size: 0.75rem; color: #0c0c0f; outline: none; min-width: 0; }
    .secret-row input:focus { border-color: #00c3ff; }
    .secret-row .hint { color: #999; font-size: 0.65rem; font-family: system-ui, -apple-system, sans-serif; text-transform: none; letter-spacing: normal; }
    @media (max-width: 600px) {
      .toolbar {
        flex-wrap: wrap;
        gap: 0.4rem;
        padding: 0.5rem 0.8rem;
      }
      .toolbar button {
        padding: 0.35rem 0.3rem;
        font-size: 0.8rem;
      }
      .secret-row {
        flex-direction: column;
        align-items: stretch;
        gap: 0.4rem;
      }
      .secret-row input {
        font-size: 0.85rem;
        padding: 0.4rem 0.5rem;
      }
      .output {
        font-size: 0.9rem;
      }
    }
  `;

  private _formats: Array<"json" | "cln" | "raw" | "rune"> = ["json", "cln", "raw", "rune"];

  render() {
    return html`
      <div class="toolbar">
        ${this._formats.map(f => html`
          <button class=${f === this.format ? "active" : ""} @click=${() => this._setFormat(f)}>${f}</button>
        `)}
        <button class="copy-btn" @click=${this._copy}>📋 Copy</button>
      </div>
      ${this.format === "rune" ? html`
        <div class="secret-row">
          <label>Secret (hex)</label>
          <input .value=${this.secret} @input=${this._onSecretInput} spellcheck="false" placeholder="64-char hex secret">
          <span class="hint">client-side only</span>
        </div>
      ` : nothing}
      ${this.error
        ? html`<div class="error">${this.error}</div>`
        : html`<div class="output">${this.format === "rune" ? this.runeOutput : this.output}</div>`}
    `;
  }

  private _setFormat(f: "json" | "cln" | "raw" | "rune") {
    this.format = f;
    this.dispatchEvent(new CustomEvent("format-change", { detail: f, bubbles: true }));
  }

  private _onSecretInput(e: InputEvent) {
    this.secret = (e.target as HTMLInputElement).value;
    this.dispatchEvent(new CustomEvent("secret-change", { detail: this.secret, bubbles: true }));
  }

  private async _copy() {
    const text = this.format === "rune" ? this.runeOutput : this.output;
    if (text) { await navigator.clipboard.writeText(text); }
  }
}

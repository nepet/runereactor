import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("rf-output")
export class RfOutput extends LitElement {
  @property() output = "";
  @property() format: "json" | "cln" | "raw" = "json";
  @property() error = "";

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
  `;

  private _formats: Array<"json" | "cln" | "raw"> = ["json", "cln", "raw"];

  render() {
    return html`
      <div class="toolbar">
        ${this._formats.map(f => html`
          <button class=${f === this.format ? "active" : ""} @click=${() => this._setFormat(f)}>${f}</button>
        `)}
        <button class="copy-btn" @click=${this._copy}>📋 Copy</button>
      </div>
      ${this.error
        ? html`<div class="error">${this.error}</div>`
        : html`<div class="output">${this.output}</div>`}
    `;
  }

  private _setFormat(f: "json" | "cln" | "raw") {
    this.format = f;
    this.dispatchEvent(new CustomEvent("format-change", { detail: f, bubbles: true }));
  }

  private async _copy() {
    if (this.output) { await navigator.clipboard.writeText(this.output); }
  }
}

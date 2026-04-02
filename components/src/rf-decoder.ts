import { LitElement, html, css, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { decodeRune } from "./wasm-bridge.js";

interface DecodedCondition {
  field: string;
  op: string;
  op_name: string;
  value: string;
}
interface DecodedRestriction {
  alternatives: DecodedCondition[];
}
interface DecodedRune {
  restrictions: DecodedRestriction[];
}

@customElement("rf-decoder")
export class RfDecoder extends LitElement {
  @state() private _input = "";
  @state() private _decoded: DecodedRune | null = null;
  @state() private _error = "";

  static styles = css`
    :host {
      display: block;
      border: 1px solid #e2e4e8;
      border-radius: 8px;
      overflow: hidden;
      background: #fff;
      font-family: system-ui, -apple-system, sans-serif;
      color: #0c0c0f;
    }

    .header {
      padding: 0.6rem 1rem;
      font-size: 0.75rem;
      font-weight: 600;
      color: #666;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      background: #f7f8fa;
      border-bottom: 1px solid #e2e4e8;
    }

    textarea {
      display: block;
      width: 100%;
      box-sizing: border-box;
      border: none;
      border-bottom: 1px solid #e2e4e8;
      padding: 0.8rem 1rem;
      font-family: "JetBrains Mono", "Fira Code", monospace;
      font-size: 0.8rem;
      line-height: 1.6;
      resize: vertical;
      min-height: 60px;
      background: #f7f8fa;
      color: #0c0c0f;
      outline: none;
    }

    textarea::placeholder {
      color: #999;
    }

    .empty {
      padding: 2rem 1rem;
      text-align: center;
      color: #999;
      font-size: 0.85rem;
    }

    .error {
      padding: 0.8rem 1rem;
      color: #dc2626;
      font-family: "JetBrains Mono", "Fira Code", monospace;
      font-size: 0.8rem;
      background: #fef2f2;
      border-bottom: 1px solid #fecaca;
    }

    .restriction-card {
      margin: 0.6rem;
      border: 1px solid #ebedef;
      border-radius: 6px;
      overflow: hidden;
    }

    .restriction-header {
      padding: 0.4rem 0.8rem;
      font-size: 0.7rem;
      font-weight: 600;
      color: #666;
      background: #f7f8fa;
      border-bottom: 1px solid #ebedef;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.8rem;
    }

    th {
      text-align: left;
      padding: 0.4rem 0.8rem;
      font-size: 0.65rem;
      font-weight: 600;
      color: #999;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      background: #f7f8fa;
      border-bottom: 1px solid #ebedef;
    }

    td {
      padding: 0.4rem 0.8rem;
      border-bottom: 1px solid #ebedef;
      font-family: "JetBrains Mono", "Fira Code", monospace;
      font-size: 0.78rem;
    }

    tr:last-child td {
      border-bottom: none;
    }

    .op {
      color: #b8960e;
      font-weight: 600;
    }

    .or-label {
      display: block;
      text-align: center;
      padding: 0.25rem 0;
      font-size: 0.7rem;
      font-weight: 700;
      color: #0088b3;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
  `;

  render() {
    return html`
      <div class="header">Rune Decoder</div>
      <textarea
        placeholder="Paste a raw rune string here to decode it..."
        .value=${this._input}
        @input=${this._onInput}
        spellcheck="false"
      ></textarea>
      ${this._error
        ? html`<div class="error">${this._error}</div>`
        : this._decoded
          ? this._renderDecoded()
          : html`<div class="empty">Paste a rune above to see its decoded restrictions.</div>`}
    `;
  }

  private _renderDecoded() {
    if (!this._decoded || this._decoded.restrictions.length === 0) {
      return html`<div class="empty">No restrictions found in this rune.</div>`;
    }
    return this._decoded.restrictions.map(
      (r, i) => html`
        <div class="restriction-card">
          <div class="restriction-header">
            Restriction #${i + 1} &mdash; ${r.alternatives.length} alternative${r.alternatives.length !== 1 ? "s" : ""}
          </div>
          <table>
            <tr><th>Field</th><th>Operator</th><th>Meaning</th><th>Value</th></tr>
            ${r.alternatives.map(
              (alt, j) => html`
                ${j > 0 ? html`<tr><td colspan="4"><span class="or-label">or</span></td></tr>` : nothing}
                <tr>
                  <td>${alt.field || "(any)"}</td>
                  <td class="op">${alt.op}</td>
                  <td>${alt.op_name}</td>
                  <td>${alt.value}</td>
                </tr>
              `
            )}
          </table>
        </div>
      `
    );
  }

  private async _onInput(e: InputEvent) {
    this._input = (e.target as HTMLTextAreaElement).value.trim();
    if (!this._input) {
      this._decoded = null;
      this._error = "";
      return;
    }
    try {
      const json = await decodeRune(this._input);
      this._decoded = JSON.parse(json) as DecodedRune;
      this._error = "";
    } catch (err) {
      this._decoded = null;
      this._error = String(err);
    }
  }
}

import { LitElement, html, css, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { decodeRune, decodeRuneBase64, verifyRune } from "./wasm-bridge.js";

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

function looksLikeBase64(s: string): boolean {
  return s.length >= 44 && /^[A-Za-z0-9_\-+/=]+$/.test(s);
}

@customElement("rf-decoder")
export class RfDecoder extends LitElement {
  @state() private _input = "";
  @state() private _decoded: DecodedRune | null = null;
  @state() private _error = "";
  @state() private _format: "raw" | "base64" | "" = "";
  @state() private _verifySecret = "";
  @state() private _verifyResult: "pass" | "fail" | "" = "";
  @state() private _verifyError = "";

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
      display: flex;
      justify-content: space-between;
      align-items: center;
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

    .format-badge { font-size: 0.65rem; font-weight: 600; padding: 0.15rem 0.5rem; border-radius: 3px; text-transform: uppercase; letter-spacing: 0.05em; }
    .format-badge.raw { background: #ebedef; color: #666; }
    .format-badge.base64 { background: #e0f7ff; color: #0088b3; }

    .verify-section { padding: 0.8rem 1rem; border-top: 1px solid #e2e4e8; background: #f7f8fa; }
    .verify-title { font-size: 0.7rem; font-weight: 600; color: #666; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.5rem; }
    .verify-row { display: flex; gap: 0.5rem; align-items: center; }
    .verify-row input { flex: 1; border: 1px solid #e2e4e8; border-radius: 4px; padding: 0.35rem 0.5rem; font-family: "JetBrains Mono", "Fira Code", monospace; font-size: 0.78rem; color: #0c0c0f; outline: none; min-width: 0; }
    .verify-row input:focus { border-color: #00c3ff; }
    .verify-btn { background: #0088b3; color: #fff; border: none; border-radius: 4px; padding: 0.35rem 0.8rem; font-size: 0.75rem; font-weight: 600; cursor: pointer; white-space: nowrap; }
    .verify-btn:hover { background: #006d91; }
    .verify-result { margin-top: 0.4rem; font-size: 0.78rem; font-weight: 600; }
    .verify-pass { color: #16a34a; }
    .verify-fail { color: #dc2626; }

    .table-scroll {
      overflow-x: auto;
    }

    @media (max-width: 600px) {
      textarea {
        font-size: 0.9rem;
      }
      th:nth-child(3),
      td:nth-child(3) {
        display: none;
      }
      .verify-row {
        flex-direction: column;
        align-items: stretch;
      }
      .verify-row input {
        font-size: 0.85rem;
        padding: 0.4rem 0.5rem;
      }
      .verify-btn {
        width: 100%;
        padding: 0.5rem 0.8rem;
      }
    }
  `;

  render() {
    return html`
      <div class="header">
        <span>Rune Decoder</span>
        ${this._format ? html`<span class="format-badge ${this._format}">${this._format}</span>` : nothing}
      </div>
      <textarea
        placeholder="Paste a raw rune string or base64-encoded rune to decode..."
        .value=${this._input}
        @input=${this._onInput}
        spellcheck="false"
      ></textarea>
      ${this._error
        ? html`<div class="error">${this._error}</div>`
        : this._decoded
          ? this._renderDecoded()
          : html`<div class="empty">Paste a rune above to see its decoded restrictions.</div>`}
      ${this._format === "base64" && this._decoded ? this._renderVerify() : nothing}
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
          <div class="table-scroll"><table>
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
          </table></div>
        </div>
      `
    );
  }

  private _renderVerify() {
    return html`
      <div class="verify-section">
        <div class="verify-title">Verify against secret</div>
        <div class="verify-row">
          <input
            placeholder="Enter hex secret to verify..."
            .value=${this._verifySecret}
            @input=${(e: InputEvent) => { this._verifySecret = (e.target as HTMLInputElement).value; this._verifyResult = ""; this._verifyError = ""; }}
            spellcheck="false"
          >
          <button class="verify-btn" @click=${this._verify}>Verify</button>
        </div>
        ${this._verifyResult === "pass" ? html`<div class="verify-result verify-pass">&#x2713; Valid &mdash; this rune was derived from the given secret</div>` : nothing}
        ${this._verifyResult === "fail" ? html`<div class="verify-result verify-fail">&#x2717; Invalid &mdash; ${this._verifyError || "rune does not match this secret"}</div>` : nothing}
      </div>
    `;
  }

  private async _onInput(e: InputEvent) {
    this._input = (e.target as HTMLTextAreaElement).value.trim();
    this._verifyResult = "";
    this._verifyError = "";

    if (!this._input) {
      this._decoded = null;
      this._error = "";
      this._format = "";
      return;
    }

    if (looksLikeBase64(this._input)) {
      try {
        const json = await decodeRuneBase64(this._input);
        this._decoded = JSON.parse(json) as DecodedRune;
        this._error = "";
        this._format = "base64";
        return;
      } catch {
        // Fall through to try raw format
      }
    }

    try {
      const json = await decodeRune(this._input);
      this._decoded = JSON.parse(json) as DecodedRune;
      this._error = "";
      this._format = "raw";
    } catch (err) {
      this._decoded = null;
      this._error = String(err);
      this._format = "";
    }
  }

  private async _verify() {
    if (!this._verifySecret || !this._input) return;
    try {
      await verifyRune(this._verifySecret, this._input);
      this._verifyResult = "pass";
      this._verifyError = "";
    } catch (err) {
      this._verifyResult = "fail";
      this._verifyError = String(err);
    }
  }
}

import { LitElement, html, css, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { generatePolicy, compilePolicy } from "./wasm-bridge.js";
import "./rf-output.js";

interface Condition {
  field: string;
  op: string;
  value: string;
}

interface WhenBlock {
  method: string;
  conditions: Condition[];
}

const OPERATORS = [
  { sym: "=", label: "equals" },
  { sym: "/", label: "not equal" },
  { sym: "!", label: "missing" },
  { sym: "<", label: "less than" },
  { sym: ">", label: "greater than" },
  { sym: "{", label: "lex less than" },
  { sym: "}", label: "lex greater than" },
  { sym: "^", label: "starts with" },
  { sym: "$", label: "ends with" },
  { sym: "~", label: "contains" },
];

const FIELDS = [
  { value: "time", hint: "Current UNIX timestamp", group: "built-in" },
  { value: "id", hint: "Node ID of the peer", group: "built-in" },
  { value: "method", hint: "Command being run", group: "built-in" },
  { value: "per", hint: "Rate limit interval (e.g. 5sec, 1min, 1hour, 1day)", group: "built-in" },
  { value: "rate", hint: "Rate limit per minute", group: "built-in" },
  { value: "pnum", hint: "Number of parameters", group: "built-in" },
  { value: "pnameamount_msat", hint: "Named param: amount_msat", group: "pname" },
  { value: "pnamedestination", hint: "Named param: destination", group: "pname" },
  { value: "pnamedescription", hint: "Named param: description", group: "pname" },
  { value: "pnamelabel", hint: "Named param: label", group: "pname" },
  { value: "pnameinvstring", hint: "Named param: invstring", group: "pname" },
  { value: "pnamebolt11", hint: "Named param: bolt11", group: "pname" },
  { value: "parr0", hint: "First positional parameter", group: "parr" },
  { value: "parr1", hint: "Second positional parameter", group: "parr" },
];

@customElement("rf-builder")
export class RfBuilder extends LitElement {
  @state() private _tagField = "";
  @state() private _tagValue = "";
  @state() private _id = "";
  @state() private _methods = "";
  @state() private _whens: WhenBlock[] = [];
  @state() private _globals: Condition[] = [];
  @state() private _rfSource = "";
  @state() private _output = "";
  @state() private _outputFormat: "json" | "cln" | "raw" = "json";
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
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .header a {
      font-size: 0.7rem;
      color: #00c3ff;
      text-decoration: none;
      font-weight: 600;
      text-transform: none;
      letter-spacing: normal;
    }

    .header a:hover {
      color: #0088b3;
    }

    .section {
      padding: 0.8rem 1rem;
      border-bottom: 1px solid #ebedef;
    }

    .section-title {
      font-size: 0.7rem;
      font-weight: 600;
      color: #666;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      margin-bottom: 0.5rem;
    }

    .row {
      display: flex;
      gap: 0.5rem;
      align-items: center;
      margin-bottom: 0.4rem;
    }

    .row:last-child {
      margin-bottom: 0;
    }

    input, select {
      border: 1px solid #e2e4e8;
      border-radius: 4px;
      padding: 0.35rem 0.5rem;
      font-size: 0.8rem;
      color: #0c0c0f;
      background: #fff;
      outline: none;
      font-family: inherit;
    }

    input:focus, select:focus {
      border-color: #00c3ff;
    }

    input.wide {
      flex: 1;
      min-width: 0;
    }

    input:disabled {
      background: #f7f8fa;
      color: #999;
    }

    .add-btn {
      background: none;
      border: 1px dashed #e2e4e8;
      border-radius: 4px;
      padding: 0.3rem 0.6rem;
      font-size: 0.75rem;
      color: #00c3ff;
      cursor: pointer;
      font-weight: 600;
    }

    .add-btn:hover {
      border-color: #00c3ff;
      background: #f0fbff;
    }

    .remove-btn {
      background: none;
      border: none;
      color: #dc2626;
      cursor: pointer;
      font-size: 0.85rem;
      padding: 0.2rem 0.4rem;
      font-weight: 600;
      line-height: 1;
    }

    .remove-btn:hover {
      background: #fef2f2;
      border-radius: 4px;
    }

    .when-block {
      border: 1px solid #ebedef;
      border-radius: 6px;
      margin-bottom: 0.5rem;
      overflow: hidden;
    }

    .when-header {
      display: flex;
      align-items: center;
      gap: 0.5rem;
      padding: 0.4rem 0.6rem;
      background: #f7f8fa;
      border-bottom: 1px solid #ebedef;
      font-size: 0.75rem;
    }

    .when-header input {
      font-size: 0.75rem;
    }

    .when-body {
      padding: 0.5rem 0.6rem;
    }

    .preview {
      padding: 0.8rem 1rem;
      border-bottom: 1px solid #ebedef;
    }

    .preview-title {
      font-size: 0.7rem;
      font-weight: 600;
      color: #666;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      margin-bottom: 0.4rem;
    }

    .preview pre {
      margin: 0;
      padding: 0.6rem 0.8rem;
      background: #f7f8fa;
      border: 1px solid #ebedef;
      border-radius: 6px;
      font-family: "JetBrains Mono", "Fira Code", monospace;
      font-size: 0.78rem;
      line-height: 1.7;
      white-space: pre-wrap;
      word-break: break-all;
      color: #0c0c0f;
      min-height: 2rem;
    }

    .error {
      color: #dc2626;
      font-size: 0.78rem;
      padding: 0.4rem 0;
      font-family: "JetBrains Mono", "Fira Code", monospace;
    }

    label {
      font-size: 0.75rem;
      color: #666;
      white-space: nowrap;
    }
  `;

  render() {
    return html`
      <div class="header">
        <span>Policy Builder</span>
        ${this._rfSource
          ? html`<a href="/playground#policy=${encodeURIComponent(this._rfSource)}">Open in Playground</a>`
          : nothing}
      </div>

      <!-- Tag -->
      <div class="section">
        <div class="section-title">Tag</div>
        <div class="row">
          <label>Field</label>
          <input class="wide" placeholder="e.g. operator_id" .value=${this._tagField} @input=${(e: InputEvent) => { this._tagField = (e.target as HTMLInputElement).value; this._generate(); }}>
          <label>Value</label>
          <input class="wide" placeholder="e.g. default-operator" .value=${this._tagValue} @input=${(e: InputEvent) => { this._tagValue = (e.target as HTMLInputElement).value; this._generate(); }}>
        </div>
      </div>

      <!-- Peer ID -->
      <div class="section">
        <div class="section-title">Peer ID</div>
        <div class="row">
          <input class="wide" placeholder="Optional 66-char hex public key" .value=${this._id} @input=${(e: InputEvent) => { this._id = (e.target as HTMLInputElement).value; this._generate(); }}>
        </div>
      </div>

      <!-- Allowed Methods -->
      <div class="section">
        <div class="section-title">Allowed Methods</div>
        <div class="row">
          <input class="wide" placeholder="Comma-separated, e.g. listfunds, xpay, invoice" .value=${this._methods} @input=${(e: InputEvent) => { this._methods = (e.target as HTMLInputElement).value; this._generate(); }}>
        </div>
      </div>

      <!-- When Blocks -->
      <div class="section">
        <div class="section-title">When Blocks</div>
        <div style="font-size:0.7rem;color:#999;margin-bottom:0.5rem">Constrain specific methods. Use <code style="font-size:0.65rem;background:#f7f8fa;padding:0.1rem 0.2rem;border-radius:2px">pnameX</code> for named parameters, <code style="font-size:0.65rem;background:#f7f8fa;padding:0.1rem 0.2rem;border-radius:2px">parrN</code> for positional.</div>
        ${this._whens.map((w, wi) => html`
          <div class="when-block">
            <div class="when-header">
              <label>Method</label>
              <input placeholder="e.g. xpay" .value=${w.method} @input=${(e: InputEvent) => { this._whens[wi].method = (e.target as HTMLInputElement).value; this._whens = [...this._whens]; this._generate(); }}>
              <button class="remove-btn" @click=${() => this._removeWhen(wi)} title="Remove when block">&times;</button>
            </div>
            <div class="when-body">
              ${w.conditions.map((c, ci) => this._renderConditionRow(c, (field, val) => {
                this._whens[wi].conditions[ci] = { ...this._whens[wi].conditions[ci], [field]: val };
                if (field === "op" && val === "!") {
                  this._whens[wi].conditions[ci].value = "";
                }
                this._whens = [...this._whens];
                this._generate();
              }, () => this._removeWhenCondition(wi, ci)))}
              <button class="add-btn" @click=${() => this._addWhenCondition(wi)}>+ Condition</button>
            </div>
          </div>
        `)}
        <button class="add-btn" @click=${this._addWhen}>+ When Block</button>
      </div>

      <!-- Global Constraints -->
      <div class="section">
        <div class="section-title">Global Constraints</div>
        ${this._globals.map((c, ci) => this._renderConditionRow(c, (field, val) => {
          this._globals[ci] = { ...this._globals[ci], [field]: val };
          if (field === "op" && val === "!") {
            this._globals[ci].value = "";
          }
          this._globals = [...this._globals];
          this._generate();
        }, () => this._removeGlobal(ci)))}
        <button class="add-btn" @click=${this._addGlobal}>+ Constraint</button>
      </div>

      <!-- Preview -->
      ${this._rfSource ? html`
        <div class="preview">
          <div class="preview-title">Generated .rf Source</div>
          <pre>${this._rfSource}</pre>
        </div>
      ` : nothing}

      ${this._error ? html`
        <div class="section">
          <div class="error">${this._error}</div>
        </div>
      ` : nothing}

      <!-- Compiled Output -->
      ${this._rfSource ? html`
        <rf-output .output=${this._output} .format=${this._outputFormat} .error=${this._error} @format-change=${this._onFormatChange}></rf-output>
      ` : nothing}
    `;
  }

  private _renderConditionRow(
    c: Condition,
    onChange: (field: string, val: string) => void,
    onRemove: () => void
  ) {
    return html`
      <div class="row">
        <select @change=${(e: Event) => {
          const val = (e.target as HTMLSelectElement).value;
          if (val !== "__custom__") {
            onChange("field", val);
          } else {
            onChange("field", "");
          }
        }}>
          <option value="" ?selected=${!c.field}>Select field...</option>
          ${FIELDS.map(f => html`<option value=${f.value} ?selected=${c.field === f.value}>${f.value} — ${f.hint}</option>`)}
          <option value="__custom__" ?selected=${c.field !== "" && !FIELDS.some(f => f.value === c.field)}>Custom...</option>
        </select>
        ${c.field !== "" && !FIELDS.some(f => f.value === c.field) ? html`
          <input placeholder="pnameX, parrN, etc." .value=${c.field} @input=${(e: InputEvent) => onChange("field", (e.target as HTMLInputElement).value)}>
        ` : nothing}
        <select .value=${c.op} @change=${(e: Event) => onChange("op", (e.target as HTMLSelectElement).value)}>
          ${OPERATORS.map(o => html`<option value=${o.sym} ?selected=${c.op === o.sym}>${o.sym} (${o.label})</option>`)}
        </select>
        <input class="wide" placeholder="value" .value=${c.value} ?disabled=${c.op === "!"} @input=${(e: InputEvent) => onChange("value", (e.target as HTMLInputElement).value)}>
        <button class="remove-btn" @click=${onRemove} title="Remove">&times;</button>
      </div>
    `;
  }

  private _addWhen() {
    this._whens = [...this._whens, { method: "", conditions: [{ field: "", op: "=", value: "" }] }];
  }

  private _removeWhen(index: number) {
    this._whens = this._whens.filter((_, i) => i !== index);
    this._generate();
  }

  private _addWhenCondition(whenIndex: number) {
    this._whens[whenIndex].conditions.push({ field: "", op: "=", value: "" });
    this._whens = [...this._whens];
  }

  private _removeWhenCondition(whenIndex: number, condIndex: number) {
    this._whens[whenIndex].conditions = this._whens[whenIndex].conditions.filter((_, i) => i !== condIndex);
    this._whens = [...this._whens];
    this._generate();
  }

  private _addGlobal() {
    this._globals = [...this._globals, { field: "", op: "=", value: "" }];
  }

  private _removeGlobal(index: number) {
    this._globals = this._globals.filter((_, i) => i !== index);
    this._generate();
  }

  private _onFormatChange(e: CustomEvent) {
    this._outputFormat = e.detail;
    this._compile();
  }

  private async _generate() {
    const methods = this._methods
      .split(",")
      .map(m => m.trim())
      .filter(m => m.length > 0);

    const spec: Record<string, unknown> = {
      tag: this._tagField || this._tagValue
        ? { field: this._tagField, value: this._tagValue }
        : null,
      id: this._id || null,
      methods,
      when: this._whens
        .filter(w => w.method)
        .map(w => ({
          method: w.method,
          conditions: w.conditions.filter(c => c.field),
        })),
      global: this._globals.filter(c => c.field),
    };

    // Only generate if there's meaningful content
    const hasContent = spec.tag || spec.id || methods.length > 0 ||
      (spec.when as unknown[]).length > 0 || (spec.global as unknown[]).length > 0;

    if (!hasContent) {
      this._rfSource = "";
      this._output = "";
      this._error = "";
      return;
    }

    try {
      this._rfSource = await generatePolicy(JSON.stringify(spec));
      this._error = "";
      this._compile();
    } catch (e) {
      this._error = String(e);
      this._rfSource = "";
      this._output = "";
    }
  }

  private async _compile() {
    if (!this._rfSource) return;
    try {
      this._output = await compilePolicy(this._rfSource, this._outputFormat);
      this._error = "";
    } catch (e) {
      this._error = String(e);
      this._output = "";
    }
  }
}

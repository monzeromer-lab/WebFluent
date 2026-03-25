export interface TemplateOptions {
  /** Theme name (default: "default") */
  theme?: string;
  /** Custom design tokens */
  tokens?: Record<string, string>;
}

export class Template {
  constructor(source: string, options?: TemplateOptions);

  /** Create a Template from a .wf source string. */
  static fromString(source: string, options?: TemplateOptions): Template;

  /** Create a Template from a .wf file path. */
  static fromFile(filePath: string, options?: TemplateOptions): Template;

  /** Set the theme. Returns this for chaining. */
  withTheme(theme: string): this;

  /** Set custom design tokens. Returns this for chaining. */
  withTokens(tokens: Record<string, string>): this;

  /** Render to a full HTML document string. */
  renderHtml(data: Record<string, unknown>): string;

  /** Render to an HTML fragment string (no <html> wrapper). */
  renderHtmlFragment(data: Record<string, unknown>): string;

  /** Render to a PDF Buffer. */
  renderPdf(data: Record<string, unknown>): Buffer;
}

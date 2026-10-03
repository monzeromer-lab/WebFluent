export interface TemplateOptions {
  /** Which `theme` the template declares to render with; only needed when it declares more than one */
  theme?: string;
  /** Design tokens over the theme's, e.g. `{ "color-primary": "#8B5CF6" }` */
  tokens?: Record<string, string>;
  /** The page to render, by name, when the template has several */
  page?: string;
  /** The language `renderHtml` declares, `<html lang>` (default `en`) */
  lang?: string;
}

export class Template {
  constructor(source: string, options?: TemplateOptions);

  /** Create a Template from a .wf source string. */
  static fromString(source: string, options?: TemplateOptions): Template;

  /** Create a Template from a .wf or .wfx file path. */
  static fromFile(filePath: string, options?: TemplateOptions): Template;

  /** Every .wf and .wfx file under a directory, as one template: components shared, pages picked with `page()`. */
  static fromDir(dirPath: string, options?: TemplateOptions): Template;

  /** The same template, rendering only the page called `name`. */
  page(name: string): Template;

  /** The language `renderHtml` declares, `<html lang>`. Returns this for chaining. */
  withLang(lang: string): this;

  /** Render with one of the `theme` declarations the template makes. Returns this for chaining. */
  withTheme(theme: string): this;

  /** Set custom design tokens. Returns this for chaining. */
  withTokens(tokens: Record<string, string>): this;

  /** Render to a full HTML document string. */
  renderHtml(data: Record<string, unknown>): string;

  /** Render to an HTML fragment string (no <html> wrapper). */
  renderHtmlFragment(data: Record<string, unknown>): string;

  /** Render to a PDF Buffer. */
  renderPdf(data: Record<string, unknown>): Buffer;

  /** Render a `Presentation` to a PDF slide deck. */
  renderSlides(data: Record<string, unknown>): Buffer;

  /** `renderHtml`, without blocking the event loop. */
  renderHtmlAsync(data: Record<string, unknown>): Promise<string>;

  /** `renderHtmlFragment`, without blocking the event loop. */
  renderHtmlFragmentAsync(data: Record<string, unknown>): Promise<string>;

  /** `renderPdf`, without blocking the event loop. */
  renderPdfAsync(data: Record<string, unknown>): Promise<Buffer>;

  /** `renderSlides`, without blocking the event loop. */
  renderSlidesAsync(data: Record<string, unknown>): Promise<Buffer>;
}

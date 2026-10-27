import { marked } from 'marked';
import DOMPurify from 'dompurify';

/** Markdown to sanitized HTML. Links open in a new tab. */
export function renderMarkdown(text: string): string {
  const html = marked.parse(text, { async: false, gfm: true, breaks: false }) as string;
  const clean = DOMPurify.sanitize(html);
  return clean.replace(/<a /g, '<a target="_blank" rel="noopener noreferrer" ');
}

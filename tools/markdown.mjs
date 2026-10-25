// Markdown → HTML (2.2.0: out of build-script-guides.mjs, shared with the
// site, tools/build-site.mjs). The subset the docs use: headings, paragraphs,
// fenced code, tables, lists, block quotes, rules, and inline code, bold,
// italic and links. Anything else stays text, escaped.
//
//   render(markdown, { ids: new Set(), toc: [] })   the HTML; `toc` collects
//                                                   the h2 and h3, `ids` the
//                                                   anchors already taken

export const escapeHtml = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
export const escapeAttr = (s) => escapeHtml(s).replace(/"/g, '&quot;');

export function slugify(text, used) {
  const base = text.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase()
    .replace(/[`*]/g, '').replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'section';
  let id = base;
  for (let n = 2; used.has(id); n++) id = `${base}-${n}`;
  used.add(id);
  return id;
}

/** Inline markdown. Code spans are set aside first: nothing inside them is markup. */
export function inline(md) {
  const codes = [];
  let s = md.replace(/`([^`]+)`/g, (_, c) => `\u0000${codes.push(c) - 1}\u0000`);
  s = escapeHtml(s);
  s = s.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, t, u) => `<a href="${u.replace(/"/g, '&quot;')}">${t}</a>`);
  s = s.replace(/\*\*((?:[^*]|\*[^*]+\*)+?)\*\*/g, '<strong>$1</strong>');
  s = s.replace(/(^|[^*\w])\*([^*\s](?:[^*]*?[^*\s])?)\*(?![*\w])/g, '$1<em>$2</em>');
  return s.replace(/\u0000(\d+)\u0000/g, (_, i) => `<code>${escapeHtml(codes[i])}</code>`);
}

const MARKERS = 'INFO|OK|WARN|ERR|STEP|PROGRESS|CKPT|REBOOT|RESTART|DONE|FREED|LOG|FIND|ITEM|METRIC|NOTE';

/** A light colouring of code: `##` metadata, comments, strings, output markers. */
function highlight(code) {
  return code.split('\n').map((line) => {
    const t = line.trimStart();
    if (t.startsWith('##')) return `<span class="k-meta">${escapeHtml(line)}</span>`;
    if (t.startsWith('#') || t.startsWith('<#')) return `<span class="k-com">${escapeHtml(line)}</span>`;
    let s = escapeHtml(line);
    s = s.replace(/"(?:[^"\\`]|`.|\\.)*"|'[^']*'/g, (m) => `<span class="k-str">${m}</span>`);
    return s.replace(new RegExp(`\\[(${MARKERS})\\]`, 'g'), '<span class="k-mark">[$1]</span>');
  }).join('\n');
}

/** Splits a table row on `|`, but never inside a code span. */
export function cells(row) {
  const r = row.trim().replace(/^\|/, '').replace(/\|$/, '');
  const out = [];
  let cur = '', inCode = false;
  for (const ch of r) {
    if (ch === '`') inCode = !inCode;
    if (ch === '|' && !inCode) { out.push(cur.trim()); cur = ''; } else cur += ch;
  }
  out.push(cur.trim());
  return out;
}

const SEVERITIES = new Set(['erreur', 'avertissement', 'note']);

export function render(src, ctx, { inQuote = false } = {}) {
  const lines = src.split('\n');
  const out = [];
  let para = [];
  const flush = () => {
    if (para.length) out.push(`<p>${inline(para.join(' '))}</p>`);
    para = [];
  };
  const marker = /^(\s*)([-*]|\d+\.)\s+(.*)$/;
  let i = 0;
  while (i < lines.length) {
    const l = lines[i];
    if (/^\s*```/.test(l)) {
      flush();
      const indent = l.length - l.trimStart().length;
      const buf = [];
      for (i++; i < lines.length && !/^\s*```\s*$/.test(lines[i]); i++) buf.push(lines[i].slice(Math.min(indent, lines[i].length - lines[i].trimStart().length)));
      i++;
      out.push(`<pre><code>${highlight(buf.join('\n'))}</code></pre>`);
      continue;
    }
    const h = /^(#{1,6})\s+(.*)$/.exec(l);
    if (h) {
      flush();
      const level = h[1].length;
      const text = h[2].trim();
      const id = slugify(text, ctx.ids);
      out.push(`<h${level} id="${id}">${inline(text)}</h${level}>`);
      if (!inQuote && (level === 2 || level === 3)) ctx.toc.push({ level, id, html: inline(text) });
      i++;
      continue;
    }
    if (/^---+\s*$/.test(l)) { flush(); out.push('<hr>'); i++; continue; }
    if (/^\s*\|/.test(l) && /^\s*\|?\s*:?-{3,}/.test(lines[i + 1] || '')) {
      flush();
      const head = cells(l);
      const rows = [];
      for (i += 2; i < lines.length && /^\s*\|/.test(lines[i]); i++) rows.push(cells(lines[i]));
      const td = (c) => (SEVERITIES.has(c) ? `<span class="sev ${c}">${c}</span>` : inline(c));
      out.push(`<div class="tbl-wrap" data-rows="${rows.length}"><table><thead><tr>${head.map((c) => `<th>${inline(c)}</th>`).join('')}</tr></thead>`
        + `<tbody>${rows.map((r) => `<tr>${r.map((c) => `<td>${td(c)}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`);
      continue;
    }
    if (/^>/.test(l)) {
      flush();
      const buf = [];
      for (; i < lines.length && /^>/.test(lines[i]); i++) buf.push(lines[i].replace(/^> ?/, ''));
      out.push(`<blockquote>${render(buf.join('\n'), ctx, { inQuote: true })}</blockquote>`);
      continue;
    }
    const m = marker.exec(l);
    if (m && m[1].length === 0) {
      flush();
      const ordered = /\d/.test(m[2]);
      const items = [];
      const numbers = [];
      while (i < lines.length) {
        const mm = marker.exec(lines[i]);
        if (!mm || mm[1].length !== 0 || /\d/.test(mm[2]) !== ordered) break;
        numbers.push(parseInt(mm[2], 10));
        const body = [mm[3]];
        for (i++; i < lines.length; i++) {
          const n = lines[i];
          if (n.trim() === '') {
            // A blank line inside an item continues it only if indented text follows.
            if (/^\s{2,}\S/.test(lines[i + 1] || '')) { body.push(''); continue; }
            break;
          }
          if (!/^\s/.test(n)) break;
          body.push(n.replace(/^\s{2,4}/, ''));
        }
        items.push(body.join('\n'));
        if ((lines[i] || '').trim() === '' && marker.exec(lines[i + 1] || '')?.[1] === '') i++;
      }
      const tag = ordered ? 'ol' : 'ul';
      out.push(`<${tag}>${items.map((b, k) => {
        const html = render(b, ctx, { inQuote: true });
        const single = /^<p>([\s\S]*)<\/p>$/.exec(html);
        // A numbered rule keeps its number (« 44. »), even alone in its list.
        const value = ordered && numbers[k] !== k + 1 ? ` value="${numbers[k]}"` : '';
        return `<li${value}>${single && !html.includes('</p><') ? single[1] : html}</li>`;
      }).join('')}</${tag}>`);
      continue;
    }
    if (l.trim() === '') { flush(); i++; continue; }
    para.push(l.trim());
    i++;
  }
  flush();
  return out.join('\n');
}

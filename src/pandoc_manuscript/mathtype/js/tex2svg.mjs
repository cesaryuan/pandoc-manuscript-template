// Render a LaTeX math payload to a standalone SVG for cross-platform MathType
// WMF previews. The SVG goes to stdout; a JSON metrics line (viewBox, ex sizes,
// vertical-align depth) goes to stderr so the Python caller can derive point
// sizes and the baseline offset. LaTeX is read from a file to avoid shell
// quoting problems with `$`, `\`, and braces.

import { readFileSync } from 'node:fs';
import { mathjax } from 'mathjax-full/js/mathjax.js';
import { TeX } from 'mathjax-full/js/input/tex.js';
import { SVG } from 'mathjax-full/js/output/svg.js';
import { liteAdaptor } from 'mathjax-full/js/adaptors/liteAdaptor.js';
import { RegisterHTMLHandler } from 'mathjax-full/js/handlers/html.js';
import { AllPackages } from 'mathjax-full/js/input/tex/AllPackages.js';

function parseArgs(argv) {
  const args = { input: null, emPt: 10 };
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--input') args.input = argv[++i];
    else if (a === '--em-pt') args.emPt = parseFloat(argv[++i]);
  }
  return args;
}

// Strip math delimiters and decide inline vs display, mirroring the Python
// `mathtype_tex_payload` shape ($...$ inline, $$...$$ display).
function normalize(payload) {
  let text = payload.trim();
  let display = false;
  if (text.startsWith('$$') && text.endsWith('$$')) { display = true; text = text.slice(2, -2); }
  else if (text.startsWith('$') && text.endsWith('$')) { text = text.slice(1, -1); }
  else if (text.startsWith('\\[') && text.endsWith('\\]')) { display = true; text = text.slice(2, -2); }
  else if (text.startsWith('\\(') && text.endsWith('\\)')) { text = text.slice(2, -2); }
  return { tex: text.trim(), display };
}

function attr(svg, name) {
  const m = svg.match(new RegExp(`${name}="([^"]*)"`));
  return m ? m[1] : null;
}

const args = parseArgs(process.argv);
const payload = args.input ? readFileSync(args.input, 'utf8') : readFileSync(0, 'utf8');
const { tex, display } = normalize(payload);

const adaptor = liteAdaptor();
RegisterHTMLHandler(adaptor);
const input = new TeX({ packages: AllPackages });
const output = new SVG({ fontCache: 'local' });
const doc = mathjax.document('', { InputJax: input, OutputJax: output });

const node = doc.convert(tex, { display });
let svg = adaptor.innerHTML(node).replace(/^.*?(<svg)/s, '$1');

if (svg.includes('data-mjx-error') || svg.includes('merror')) {
  process.stderr.write(JSON.stringify({ error: `MathJax could not parse: ${tex}` }) + '\n');
  process.exit(2);
}

// MathJax emits ex-based width/height and a vertical-align depth in ex; the
// viewBox is in internal units where 1em = 1000 units.
const widthEx = parseFloat(attr(svg, 'width'));   // e.g. "10.552ex"
const heightEx = parseFloat(attr(svg, 'height'));
const style = attr(svg, 'style') || '';
const vaMatch = style.match(/vertical-align:\s*(-?[0-9.]+)ex/);
const valignEx = vaMatch ? parseFloat(vaMatch[1]) : 0;
const vb = (attr(svg, 'viewBox') || '').split(/\s+/).map(Number);
const [, minY, vbW, vbH] = vb.length === 4 ? vb : [0, 0, null, null];

// Convert to points using em = --em-pt. width_em = vbW/1000, and the depth below
// the baseline is (vbH + minY)/1000 em (viewBox min-y is negative above the line).
const emPt = args.emPt || 10;
const widthPt = vbW != null ? (vbW / 1000) * emPt : null;
const heightPt = vbH != null ? (vbH / 1000) * emPt : null;
const baselineFromBottomPt = vbW != null ? ((vbH + minY) / 1000) * emPt : null;

// Rewrite the root width/height to explicit points so LibreOffice imports the
// SVG at the intended physical size (its WMF placeable header derives from it).
if (widthPt != null && heightPt != null) {
  svg = svg
    .replace(/width="[0-9.]+ex"/, `width="${widthPt.toFixed(3)}pt"`)
    .replace(/height="[0-9.]+ex"/, `height="${heightPt.toFixed(3)}pt"`)
    .replace(/style="[^"]*vertical-align[^"]*"/, 'style=""');
}

process.stderr.write(JSON.stringify({
  tex, display,
  width_ex: widthEx, height_ex: heightEx, valign_ex: valignEx,
  em_pt: emPt,
  width_pt: widthPt, height_pt: heightPt,
  baseline_from_bottom_pt: baselineFromBottomPt,
}) + '\n');
process.stdout.write(svg);

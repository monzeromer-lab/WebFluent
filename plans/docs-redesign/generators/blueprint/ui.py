import html, json, re
E=html.escape
MONO="'JetBrains Mono', ui-monospace, monospace"
OPT={1:'Four Rooms',2:'Live Tour',3:'Pick Your Output',4:'Coming From',5:'Registry Atlas',6:'Compiler as Guide',7:'One Long Book',8:'Recipe-First',9:'Source and Output',10:'Agent-Native'}
def st(**k): return '; '.join(f"{a.replace('_','-')}: {b}" for a,b in k.items())
def tag(t,c='',attrs='',**k):
    s=f' style="{st(**k)}"' if k else ''
    return f'<{t}{attrs}{s}>{c}</{t}>'
def div(c='',**k): return tag('div',c,**k)
def txt(t,el='p',**k): k.setdefault('margin','0'); return tag(el,t,**k)
def row(c,gap='12px',**k):
    k.setdefault('align_items','center'); return div(c,display='flex',gap=gap,**k)
def col(c,gap='12px',**k): return div(c,display='flex',flex_direction='column',gap=gap,**k)
def wrap(c,gap='12px',**k): k.setdefault('align_items','center'); return div(c,display='flex',flex_wrap='wrap',gap=gap,**k)
def card(c,**k):
    d=dict(background='var(--surface)',border='1px solid var(--line)',border_radius='10px'); d.update(k); return div(c,**d)
def a(t,href='#',attrs_current=False,**k):
    d=dict(color='var(--ink)',text_decoration='none',font_size='14px'); d.update(k); cur=' aria-current="page"' if attrs_current else ''
    return f'<a href="{href}"{cur} style="{st(**d)}">{t}</a>'
def btn(t,kind='default',attrs='',**k):
    base=dict(font='600 14px Inter, sans-serif',padding='10px 16px',border_radius='6px',min_height='44px',cursor='pointer',display='inline-flex',align_items='center',gap='8px',text_decoration='none',white_space='nowrap')
    if kind=='primary': base.update(background='var(--brand)',color='#fff',border='1px solid var(--brand)')
    elif kind=='ghost': base.update(background='transparent',color='var(--ink)',border='1px solid transparent')
    else: base.update(background='var(--surface)',color='var(--ink)',border='1px solid var(--line)')
    base.update(k); return f'<button type="button"{attrs} style="{st(**base)}">{t}</button>'
def abtn(t,href,kind='default',**k):
    base=dict(font='600 14px Inter, sans-serif',padding='10px 16px',border_radius='6px',min_height='44px',display='inline-flex',align_items='center',gap='8px',text_decoration='none',white_space='nowrap',box_sizing='border-box')
    if kind=='primary': base.update(background='var(--brand)',color='#fff',border='1px solid var(--brand)')
    else: base.update(background='var(--surface)',color='var(--ink)',border='1px solid var(--line)')
    base.update(k); return f'<a href="{href}" style="{st(**base)}">{t}</a>'
def chip(t,tone='plain',**k):
    tones={'plain':('var(--sunken)','var(--muted)','var(--line)'),'brand':('var(--brand-subtle)','var(--brand-ink)','transparent'),'accent':('var(--accent-subtle)','var(--accent-ink)','transparent'),'danger':('var(--danger-subtle)','var(--danger)','transparent'),'off':('transparent','var(--muted)','var(--line)')}
    bg,fg,bd=tones[tone]
    d=dict(display='inline-flex',align_items='center',gap='6px',padding='3px 9px',border_radius='999px',font='600 12px Inter, sans-serif',background=bg,color=fg,border='1px solid '+bd,white_space='nowrap'); d.update(k)
    return tag('span',t,**d)
def mono(t,**k):
    d=dict(font='500 0.92em '+MONO,background='var(--sunken)',border='1px solid var(--line)',border_radius='4px',padding='1px 5px'); d.update(k); return tag('code',E(t),**d)
def label(t): return txt(t,font='600 13px Inter, sans-serif',color='var(--muted)')
def mode(m):
    tone={'Tutorial':'brand','How-to':'accent','Reference':'plain','About':'plain','Error':'danger'}[m]; return chip(m,tone)
def note(*ns,text=None):
    return ''
def _old_note(*ns,text=None):
    nums=' + '.join(f'{n} · {OPT[n]}' for n in ns)
    body=nums+(f' — {text}' if text else '')
    return f'<sc-if value="{{{{showNotes}}}}" hint-placeholder-val="{{{{true}}}}">'+tag('span',body,role='note',display='inline-flex',align_items='center',padding='4px 10px',border_radius='999px',background='var(--note-bg)',color='var(--note-ink)',font='600 12px Inter, sans-serif',border='1px dashed var(--note-ink)',width='fit-content',max_width='100%',line_height='1.35')+'</sc-if>'
def icon(name,size=16):
    P={'search':'<circle cx="11" cy="11" r="7"></circle><path d="m20 20-3.5-3.5"></path>','copy':'<rect x="9" y="9" width="11" height="11" rx="2"></rect><path d="M5 15V5a2 2 0 0 1 2-2h10"></path>','moon':'<path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"></path>','play':'<path d="M7 4v16l13-8z"></path>','arrow':'<path d="M5 12h14M13 6l6 6-6 6"></path>','back':'<path d="M19 12H5M11 6l-6 6 6 6"></path>','check':'<path d="M5 12l5 5L20 7"></path>','x':'<path d="M6 6l12 12M18 6 6 18"></path>','doc':'<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"></path><path d="M14 3v5h5"></path>','bot':'<rect x="4" y="8" width="16" height="12" rx="3"></rect><path d="M12 4v4M9 14h.01M15 14h.01"></path>','git':'<circle cx="6" cy="6" r="2"></circle><circle cx="6" cy="18" r="2"></circle><circle cx="18" cy="8" r="2"></circle><path d="M6 8v8M18 10c0 4-6 3-10 7"></path>','alert':'<circle cx="12" cy="12" r="9"></circle><path d="M12 8v5M12 16h.01"></path>','book':'<path d="M4 4h7a3 3 0 0 1 3 3v13a2 2 0 0 0-2-2H4z"></path><path d="M20 4h-7a3 3 0 0 0-3 3v13a2 2 0 0 1 2-2h8z"></path>'}
    return f'<svg aria-hidden="true" width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{P[name]}</svg>'
# ---- syntax ----
KW=r'\b(page|component|store|state|derived|action|effect|if|else|for|in|by|match|on|use|resource|let|await|return|validate|persist|type|enum|event|emit|slot|loading|ready|error|try|catch|required|email|test|expect|click)\b'
def hl(src,lang='wf'):
    if lang=='wf':
        pat=re.compile(r'(?P<c>//[^\n]*)|(?P<s>"(?:\\.|[^"\\])*")|(?P<k>'+KW+r')|(?P<f>\b[A-Z][A-Za-z0-9]*\b)|(?P<g>\.[a-z][A-Za-z0-9]*)|(?P<n>\b\d+(?:\.\d+)?\b)')
    elif lang=='html':
        pat=re.compile(r'(?P<c><!--.*?-->)|(?P<s>"[^"]*")|(?P<k></?[a-z][a-z0-9-]*|/?>)|(?P<f>\b[a-z-]+(?==))')
    elif lang=='js':
        pat=re.compile(r'(?P<c>//[^\n]*)|(?P<s>"(?:\\.|[^"\\])*"|`[^`]*`)|(?P<k>\b(const|let|function|return|if|else|new|await|async)\b)|(?P<f>\bWF\.[a-z]+\b)|(?P<n>\b\d+\b)')
    elif lang=='css':
        pat=re.compile(r'(?P<c>/\*.*?\*/)|(?P<k>[.#][a-z][\w-]*)|(?P<f>[a-z-]+(?=:))|(?P<s>var\([^)]*\))')
    else:
        pat=re.compile(r'(?P<c>#[^\n]*)|(?P<s>"[^"]*")|(?P<k>\bwf\b)')
    col={'c':'#959BA1','s':'#7FD6A8','k':'#8FBDE7','f':'#6FD3E6','g':'#E0B070','n':'#E0B070'}
    out=[];i=0
    for m in pat.finditer(src):
        out.append(E(src[i:m.start()])); g=m.lastgroup
        g = g if g in col else next(k for k in col if m.group(k))
        out.append(f'<span style="color: {col[g]}">{E(m.group(0))}</span>'); i=m.end()
    out.append(E(src[i:])); return ''.join(out)
def pre(src,lang='wf',**k):
    d=dict(margin='0',padding='16px 18px',background='var(--term)',color='#E6E9EC',font='13px/1.65 '+MONO,white_space='pre',overflow_x='auto',border_radius='0'); d.update(k)
    return tag('pre',hl(src,lang),**d)
def codeblock(bid,tabs,title='',extra=''):
    # tabs: list of (key,label,html_panel)
    heads=''
    for key,lab,_ in tabs:
        on=tag('button',lab,attrs=f' type="button" aria-pressed="true" onClick="{{{{tabs.{bid}.pick_{key}}}}}"',font='600 12.5px Inter, sans-serif',padding='10px 12px',background='transparent',color='#E6E9EC',border='0',border_bottom='2px solid #6FD3E6',cursor='pointer',min_height='40px')
        off=tag('button',lab,attrs=f' type="button" aria-pressed="false" onClick="{{{{tabs.{bid}.pick_{key}}}}}"',font='500 12.5px Inter, sans-serif',padding='10px 12px',background='transparent',color='#959BA1',border='0',border_bottom='2px solid transparent',cursor='pointer',min_height='40px')
        heads+=f'<sc-if value="{{{{tabs.{bid}.{key}}}}}" hint-placeholder-val="{{{{{ "true" if key==tabs[0][0] else "false" }}}}}">{on}</sc-if><sc-if value="{{{{tabs.{bid}.not_{key}}}}}" hint-placeholder-val="{{{{{ "false" if key==tabs[0][0] else "true" }}}}}">{off}</sc-if>'
    panels=''.join(f'<sc-if value="{{{{tabs.{bid}.{key}}}}}" hint-placeholder-val="{{{{{ "true" if i==0 else "false" }}}}}">{p}</sc-if>' for i,(key,_,p) in enumerate(tabs))
    top=row(row((tag('span',title,font='12px '+MONO,color='#959BA1',padding_right='10px') if title else '')+heads,gap='2px',flex_wrap='wrap')+row(extra+btn(icon('copy',14)+'Copy',kind='ghost',color='#E6E9EC',font='600 12px Inter, sans-serif',min_height='36px',padding='6px 10px',attrs=' aria-label="Copy code"'),gap='6px'),justify_content='space-between',padding='0 8px 0 14px',border_bottom='1px solid #2C3035',flex_wrap='wrap')
    return div(top+panels,background='var(--term)',border_radius='10px',overflow='hidden',border='1px solid #2C3035')
def preview_panel(inner,**k):
    d=dict(padding='28px',background_color='var(--surface)',min_height='120px',display='flex',align_items='center',justify_content='center',gap='12px',flex_wrap='wrap'); d.update(k); return tag('div',inner,attrs=' class="grid"',**d)
def _old_pp(inner,**k):
    d=dict(padding='28px',background='var(--surface)',min_height='120px',display='flex',align_items='center',justify_content='center',gap='12px',flex_wrap='wrap'); d.update(k); return div(inner,**d)
# ---- shell ----
NAV=[('Learn','Unified-Learn.dc.html'),('Recipes','Unified-Recipe.dc.html'),('Reference','Unified-Reference.dc.html'),('Errors','Unified-Error.dc.html'),('Playground','Unified-Playground.dc.html'),('Showcase','#')]
def logo(): return a(row('<svg role="img" aria-label="WebFluent" width="28" height="28" viewBox="0 0 64 64"><rect width="64" height="64" rx="12" fill="#2B5178"></rect><path fill="#FFFFFF" d="m 13.805986,45.5 -3.071,-20.35 h 4.255 l 1.591,11.1 q 0.185,1.406 0.296,2.997 0.111,1.591 0.185,2.664 0.074,-1.073 0.185,-2.664 0.111,-1.591 0.37,-2.997 l 1.961,-11.1 h 4.218 l 1.887,11.1 q 0.259,1.406 0.407,3.034 0.148,1.591 0.222,2.664 0.074,-1.073 0.148,-2.664 0.074,-1.628 0.296,-3.034 l 1.665,-11.1 h 4.144 l -3.108,20.35 h -5.55 l -1.665,-11.47 q -0.148,-1.073 -0.333,-2.553 -0.185,-1.48 -0.259,-2.442 0,0.629 -0.111,1.554 -0.111,0.925 -0.222,1.85 -0.111,0.888 -0.222,1.591 l -1.702,11.47 z M 38.28,45.5 V 32.92 H 32.878 V 27.925 H 38.28 V 24.78 q 0,-2.849 1.924,-4.551 1.961,-1.739 5.106,-1.739 h 5.957 v 4.81 h -5.772 q -0.74,0 -1.221,0.481 -0.444,0.444 -0.444,1.184 v 2.96 h 7.437 V 32.92 H 43.83 V 45.5 Z"></path></svg>'+txt('WebFluent',font_weight='700',font_size='16px',letter_spacing='-.01em')+chip('5.12'),gap='10px'),'Unified-Home.dc.html')
def header(active=None):
    nav=''.join(a(t,h,padding='9px 12px',border_radius='4px 4px 0 0',font_weight='600' if t==active else '500',color='var(--ink)' if t==active else 'var(--muted)',background='var(--surface)' if t==active else 'transparent',border_top='2px solid '+('var(--accent-ink)' if t==active else 'transparent'),attrs_current=t==active) for t,h in NAV)
    srch=a(row(icon('search',15)+tag('span','Search the docs',flex='1',white_space='nowrap',overflow='hidden',text_overflow='ellipsis')+tag('kbd','⌘K',font='11px '+MONO,border='1px solid var(--line)',border_radius='4px',padding='1px 6px',background='var(--surface)'),gap='8px'),'Unified-Search.dc.html',flex='1 1 240px',max_width='360px',min_height='40px',display='flex',align_items='center',padding='0 12px',border='1px solid var(--line)',border_radius='8px',background='var(--sunken)',color='var(--muted)',font_size='13px',box_sizing='border-box')
    out=f'<label style="{st(display="inline-flex",align_items="center",gap="6px",font_size="12px",color="var(--muted)")}"><span class="hide-sm">Building</span><select aria-label="What you are building" style="{st(font="600 13px Inter, sans-serif",padding="8px 10px",min_height="40px",border="1px solid var(--line)",border_radius="8px",background="var(--surface)",color="var(--ink)")}"><option>Website</option><option>Web app</option><option>PDF</option><option>Slides</option><option>Custom elements</option><option>Desktop / phone</option></select></label>'
    dark=tag('button',icon('moon'),attrs=' type="button" aria-label="Toggle dark mode" onClick="{{toggleDark}}"',width='40px',height='40px',display='inline-flex',align_items='center',justify_content='center',border='1px solid var(--line)',border_radius='8px',background='var(--surface)',color='var(--ink)',cursor='pointer')
    gh=a(icon('git',16)+'<span class="hide-sm">GitHub</span>','#',display='inline-flex',gap='6px',align_items='center',font_weight='600',min_height='40px',padding='0 10px')
    return tag('header',div(row(logo()+tag('nav',nav,attrs=' aria-label="Main"',display='flex',gap='2px',flex_wrap='wrap'),gap='22px',flex_wrap='wrap')+srch+row(out+dark+gh,gap='8px'),display='flex',align_items='center',justify_content='space-between',gap='16px',flex_wrap='wrap',max_width='1360px',margin='0 auto',padding='12px 24px'),border_bottom='1px solid var(--line)',background='var(--bg)')
def footer():
    cols=[('Learn',['The tour','The book','About essays']),('Do',['Recipes','Coming from…','Showcase']),('Look up',['Components','CLI and config','Errors']),('For agents',['llms.txt','wf context','MCP endpoint'])]
    c=''.join(col(label(h)+''.join(a(x,color='var(--muted)',font_size='13px') for x in xs),gap='8px',flex='1 1 160px') for h,xs in cols)
    return tag('footer',div(wrap(c,gap='24px',align_items='flex-start')+row(txt('WebFluent 5.12. Every example on this site is compiled by the release it documents.',font_size='12.5px',color='var(--muted)')+note(6,9,text='docs are built and tested with the compiler'),justify_content='space-between',flex_wrap='wrap',margin_top='28px'),max_width='1360px',margin='0 auto',padding='40px 24px'),border_top='1px solid var(--line)',background='var(--surface)',margin_top='64px')
def copymenu():
    return row(btn(icon('copy',14)+'Copy page',font_size='13px',min_height='36px',padding='6px 12px')+btn(icon('bot',14)+'.md',kind='ghost',font_size='13px',min_height='36px',padding='6px 10px',attrs=' aria-label="Open the Markdown twin"'),gap='4px')
CSSV='''.wf-root{--bg:#0E1012;--surface:#17191C;--sunken:#1F2225;--ink:#EEF0F2;--muted:#A9AEB4;--line:#2C3035;--brand:#3D6FA3;--brand-ink:#8FBDE7;--brand-subtle:#16283C;--accent-ink:#6FD3E6;--accent-subtle:#10313A;--term:#101315;--danger:#FF8A80;--danger-subtle:#3A1714;--ok:#5FD3A4;--ok-subtle:#0F2E22;--grid:rgba(143,189,231,.07)}
.wf-root.day{--bg:#F9FAFB;--surface:#FFFFFF;--sunken:#F4F6F7;--ink:#17191C;--muted:#5A5F64;--line:#E2E5E8;--brand:#2B5178;--brand-ink:#2B5178;--brand-subtle:#E7EEF6;--accent-ink:#0D6274;--accent-subtle:#E0F2F6;--term:#1B1F22;--danger:#B3261E;--danger-subtle:#FBE9E7;--ok:#1C7A5A;--ok-subtle:#E3F3EC;--grid:rgba(43,81,120,.07)}
.wf-root *{box-sizing:border-box}
.wf-root a:focus-visible,.wf-root button:focus-visible,.wf-root select:focus-visible,.wf-root input:focus-visible{outline:2px solid var(--accent-ink);outline-offset:2px}
.wf-root .grid{background-image:linear-gradient(var(--grid) 1px,transparent 1px),linear-gradient(90deg,var(--grid) 1px,transparent 1px);background-size:24px 24px;background-position:-1px -1px}
@media (max-width:760px){.hide-sm{display:none !important}}
.wf-root p a,.wf-root td a,.wf-root li a{color:var(--brand-ink)}
.wf-root p code,.wf-root li code{font-family:"JetBrains Mono",ui-monospace,monospace;font-size:.86em;background:var(--sunken);border:1px solid var(--line);border-radius:3px;padding:1px 5px}'''
FONTS='''@font-face{font-family:"Inter";font-weight:400 800;src:url("/_blob/d19d616a9bd5d5492d89a2df48f34586") format("woff2")}
@font-face{font-family:"JetBrains Mono";font-weight:400 600;src:url("/_blob/384893c6de26fed90e3face153cb051c") format("woff2")}'''
JS='''class Component extends DCLogic {
renderVals() {
const s = this.state || {};
const P = this.props;
const dark = s.dark ?? (P.dark ?? true);
const BLOCKS = __BLOCKS__;
const tabs = {};
for (const id of Object.keys(BLOCKS)) {
  const names = BLOCKS[id];
  const cur = (s.tabs && s.tabs[id]) || names[0];
  const o = {};
  for (const n of names) {
    o[n] = cur === n;
    o['not_' + n] = cur !== n;
    o['pick_' + n] = () => this.setState({ tabs: Object.assign({}, (this.state && this.state.tabs) || {}, { [id]: n }) });
  }
  tabs[id] = o;
}
const count = s.count ?? 3;
const quiz = s.quiz ?? -1;
const checked = !!s.checked;
const tone = s.tone ?? 'primary';
const size = s.size ?? 'md';
const pill = !!s.pill;
const TONES = { primary: ['#3D6FA3', '#FFFFFF'], secondary: ['#2F9FB6', '#0B0D0F'], success: ['#2F9D76', '#FFFFFF'], danger: ['#D94F45', '#FFFFFF'] };
const outlined = !!s.outlined;
const measure = !!s.measure;
const toneNames = ['primary', 'secondary', 'success', 'danger'];
const sizeNames = ['sm', 'md', 'lg'];
const SIZES = { sm: ['6px 12px', '13px', '36px'], md: ['10px 18px', '14px', '44px'], lg: ['14px 24px', '16px', '52px'] };
const t = TONES[tone] || TONES.primary;
const z = SIZES[size] || SIZES.md;
const broken = !!s.broken;
const quizOpt = (i) => () => this.setState({ quiz: i, checked: false });
return {
  rootClass: dark ? 'wf-root' : 'wf-root day',
  toggleDark: () => this.setState({ dark: !dark }),
  tabs,
  count,
  inc: () => this.setState({ count: count + 1 }),
  reset: () => this.setState({ count: 0 }),
  pick0: quizOpt(0), pick1: quizOpt(1), pick2: quizOpt(2),
  check: () => this.setState({ checked: true }),
  quizRight: checked && quiz === 1,
  quizWrong: checked && quiz !== 1,
  setTone: (e) => this.setState({ tone: e.target.value }),
  setSize: (e) => this.setState({ size: e.target.value }),
  togglePill: () => this.setState({ pill: !pill }),
  prevStyle: 'background: ' + (outlined ? 'transparent' : t[0]) + '; color: ' + (outlined ? t[0] : t[1]) + '; border: 1px solid ' + t[0] + '; padding: ' + z[0] + '; font: 600 ' + z[1] + ' Instrument Sans, Inter, sans-serif; height: ' + z[2] + '; border-radius: ' + (pill ? '999px' : '4px') + '; cursor: pointer; display: inline-flex; align-items: center',
  snippet: 'Button("Save changes").' + tone + (size !== 'md' ? '.' + size : '') + (outlined ? '.outlined' : '') + (pill ? '.pill' : ''),
  tones: toneNames.map((n) => ({ name: n, label: '.' + n, on: tone === n, off: tone !== n, pick: () => this.setState({ tone: n }) })),
  sizes: sizeNames.map((n) => ({ name: n, label: '.' + n, on: size === n, off: size !== n, pick: () => this.setState({ size: n }) })),
  outlined, notOutlined: !outlined, toggleOutlined: () => this.setState({ outlined: !outlined }),
  measure, notMeasure: !measure, toggleMeasure: () => this.setState({ measure: !measure }),
  heightLabel: z[2].replace('px', ''),
  padLabel: z[0].split(' ')[1].replace('px', ''),
  radiusLabel: pill ? 'full' : '4',
  notPill: !pill,
  pill,
  broken,
  notBroken: !broken,
  breakIt: () => this.setState({ broken: true }),
  fixIt: () => this.setState({ broken: false }),
  device: s.device ?? 'desktop',
  isDesktop: (s.device ?? 'desktop') === 'desktop',
  isPhone: (s.device ?? 'desktop') === 'phone',
  toDesktop: () => this.setState({ device: 'desktop' }),
  toPhone: () => this.setState({ device: 'phone' })
};
}
}'''
def page(fn,title,body,w,h,blocks=None,dark=False):
    props={"dark":{"editor":"boolean","default":True,"section":"Theme"},"$preview":{"width":w,"height":h}}
    doc=f'''<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{E(title)}</title>
<script src="./support.js"></script>
</head>
<body>
<x-dc>
<helmet>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Instrument+Sans:wght@400..700&amp;display=swap">
<style>
{FONTS}
body{{margin:0}}
a{{color:#2B5178}}a:hover{{opacity:.85}}
{CSSV}
</style>
</helmet>
<div class="{{{{rootClass}}}}" style="background: var(--bg); color: var(--ink); font-family: Inter, system-ui, sans-serif; min-height: 100vh">
{body}
</div>
</x-dc>
<script type="text/x-dc" data-dc-script data-props='{json.dumps(props).replace("'","&#39;")}'>
{JS.replace('__BLOCKS__',json.dumps(blocks or {}))}
</script>
</body>
</html>
'''
    open('root/project/'+fn,'w').write(doc)
    return doc

import html, json, re
E=html.escape
def st(**k): return '; '.join(f"{a.replace('_','-')}: {b}" for a,b in k.items())
def tag(t,c='',attrs='',**k):
    s=f' style="{st(**k)}"' if k else ''
    return f'<{t}{attrs}{s}>{c}</{t}>'
def div(c='',**k): return tag('div',c,**k)
def row(c,gap='12px',**k): k.setdefault('align_items','center'); return div(c,display='flex',gap=gap,**k)
def col(c,gap='12px',**k): return div(c,display='flex',flex_direction='column',gap=gap,**k)
def wrap(c,gap='12px',**k): k.setdefault('align_items','center'); return div(c,display='flex',flex_wrap='wrap',gap=gap,**k)
def lum(h):
    h=h.lstrip('#'); r,g,b=[int(h[i:i+2],16)/255 for i in (0,2,4)]
    f=lambda c: c/12.92 if c<=0.03928 else ((c+0.055)/1.055)**2.4
    return 0.2126*f(r)+0.7152*f(g)+0.0722*f(b)
def cr(a,b):
    la,lb=sorted([lum(a),lum(b)],reverse=True); return (la+0.05)/(lb+0.05)
LOGO='<svg role="img" aria-label="WebFluent" width="{s}" height="{s}" viewBox="0 0 64 64"><rect width="64" height="64" rx="12" fill="#2B5178"></rect><path fill="#FFFFFF" d="m 13.805986,45.5 -3.071,-20.35 h 4.255 l 1.591,11.1 q 0.185,1.406 0.296,2.997 0.111,1.591 0.185,2.664 0.074,-1.073 0.185,-2.664 0.111,-1.591 0.37,-2.997 l 1.961,-11.1 h 4.218 l 1.887,11.1 q 0.259,1.406 0.407,3.034 0.148,1.591 0.222,2.664 0.074,-1.073 0.148,-2.664 0.074,-1.628 0.296,-3.034 l 1.665,-11.1 h 4.144 l -3.108,20.35 h -5.55 l -1.665,-11.47 q -0.148,-1.073 -0.333,-2.553 -0.185,-1.48 -0.259,-2.442 0,0.629 -0.111,1.554 -0.111,0.925 -0.222,1.85 -0.111,0.888 -0.222,1.591 l -1.702,11.47 z M 38.28,45.5 V 32.92 H 32.878 V 27.925 H 38.28 V 24.78 q 0,-2.849 1.924,-4.551 1.961,-1.739 5.106,-1.739 h 5.957 v 4.81 h -5.772 q -0.74,0 -1.221,0.481 -0.444,0.444 -0.444,1.184 v 2.96 h 7.437 V 32.92 H 43.83 V 45.5 Z"></path></svg>'
def logo(s=32): return LOGO.replace('{s}',str(s))
# The fixed brand palette (site/src/theme.wf, Day and Night)
DAY=dict(bg='#F9FAFB',canvas='#EEF0F2',surface='#FFFFFF',sunken='#F4F6F7',ink='#17191C',muted='#5A5F64',subtle='#676C71',line='#E2E5E8',strong='#868B91',brand='#2B5178',brand_ink='#2B5178',brand_subtle='#E7EEF6',accent='#0F7285',accent_ink='#0D6274',accent_subtle='#E0F2F6',danger='#B3261E',danger_subtle='#FBE9E7',ok='#1C7A5A',warn='#8A5800',term='#1B1F22')
NIGHT=dict(bg='#0E1012',canvas='#0E1012',surface='#17191C',sunken='#1F2225',ink='#EEF0F2',muted='#A9AEB4',subtle='#878D93',line='#2C3035',strong='#6B7178',brand='#3D6FA3',brand_ink='#8FBDE7',brand_subtle='#16283C',accent='#2F9FB6',accent_ink='#6FD3E6',accent_subtle='#10313A',danger='#D94F45',danger_subtle='#3A1714',ok='#2F9D76',warn='#C98A1F',term='#101315')
SYN=dict(kw='#8FBDE7',str='#7FD6A8',num='#E0B070',fn='#6FD3E6',com='#959BA1',ink='#E6E9EC')
def vars_css(sel,P): return sel+'{'+''.join(f'--{k.replace("_","-")}:{v};' for k,v in P.items())+'}'
KW=r'\b(page|component|store|state|derived|action|if|else|for|in|by|match|on|use|resource|let|return|validate)\b'
def hl(src):
    pat=re.compile(r'(?P<c>//[^\n]*)|(?P<s>"(?:\\.|[^"\\])*")|(?P<k>'+KW+r')|(?P<f>\b[A-Z][A-Za-z0-9]*\b)|(?P<g>\.[a-z][A-Za-z0-9]*)|(?P<n>\b\d+\b)')
    col={'c':SYN['com'],'s':SYN['str'],'k':SYN['kw'],'f':SYN['fn'],'g':SYN['num'],'n':SYN['num']}
    out=[];i=0
    for m in pat.finditer(src):
        out.append(E(src[i:m.start()])); g=m.lastgroup
        out.append(f'<span style="color: {col[g]}">{E(m.group(0))}</span>'); i=m.end()
    out.append(E(src[i:])); return ''.join(out)
SAMPLE='''page Home(path: "/", title: "Hello") {
    state count = 0
    Heading("Hello, WebFluent").h1
    Button("+1").primary {
        on click { count = count + 1 }
    }
}'''
def page(fn,title,body,w,h,fonts_link='',dark=False,extra_css=''):
    FONTS='''@font-face{font-family:"Inter";font-weight:400 800;src:url("/_blob/d19d616a9bd5d5492d89a2df48f34586") format("woff2")}
@font-face{font-family:"JetBrains Mono";font-weight:400 800;src:url("/_blob/384893c6de26fed90e3face153cb051c") format("woff2")}'''
    css=vars_css('.ds',NIGHT)+vars_css('.ds.day',DAY)+'.ds *{box-sizing:border-box}.ds a:focus-visible,.ds button:focus-visible,.ds input:focus-visible,.ds select:focus-visible{outline:2px solid var(--accent-ink);outline-offset:2px}'+extra_css
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
{fonts_link}
<style>
{FONTS}
body{{margin:0}}
a{{color:#2B5178}}a:hover{{color:#1E3A57}}
{css}
</style>
</helmet>
<div class="{{{{rootClass}}}}" style="width: {w}px; min-height: {h}px; background: var(--bg); color: var(--ink)">
{body}
</div>
</x-dc>
<script type="text/x-dc" data-dc-script data-props='{json.dumps(props)}'>
class Component extends DCLogic {{
renderVals() {{
const s = this.state || {{}};
const dark = s.dark ?? (this.props.dark ?? true);
return {{
  rootClass: dark ? 'ds' : 'ds day',
  toggleDark: () => this.setState({{ dark: !dark }}),
  modeLabel: dark ? 'Switch to Day' : 'Switch to Night'
}};
}}
}}
</script>
</body>
</html>
'''
    open('root/project/'+fn,'w').write(doc); return body

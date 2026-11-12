import pages
from pages import *
from ui import hl
BROKEN_SRC=TOUR_SRC.replace('Text("Clicked {count} times")','Text("Clicked {cout} times")')
def editor_body(src,bad_line=None,bad_word=None):
    lines=src.split('\n')
    gut=''.join(f'<div>{i+1}</div>' for i in range(len(lines)))
    code=hl(src)
    if bad_word:
        code=code.replace(bad_word,f'<span style="text-decoration: underline wavy #FF6B60; text-underline-offset: 3px">{bad_word}</span>',1)
    return div(div(gut,padding='18px 10px 18px 14px',font='13px/1.65 '+MONO,color='#5F666D',text_align='right',user_select='none',border_right='1px solid #2C3035',flex='none')
        +tag('pre',code,margin='0',padding='18px 18px',font='13px/1.65 '+MONO,color='#E6E9EC',white_space='pre',overflow_x='auto',flex='1',min_width='0'),display='flex',background='var(--term)',min_width='0')
def status_bar(extra=''):
    ok=row(tag('span','',width='8px',height='8px',border_radius='4px',background='#5FD3A4',display='inline-block')+tag('span','Compiled in your browser · WebAssembly',color='#B3BAC0'),gap='8px')
    bad=row(tag('span','',width='8px',height='8px',border_radius='4px',background='#FF6B60',display='inline-block')+tag('span','1 error · the preview keeps the last good page',color='#FFB4AE'),gap='8px')
    btns=row('<sc-if value="{{notBroken}}" hint-placeholder-val="{{true}}">'+tag('button','Break it',attrs=' type="button" onClick="{{breakIt}}"',font='600 12px Inter, sans-serif',padding='6px 10px',min_height='32px',border_radius='6px',background='transparent',color='#E6E9EC',border='1px solid #3A4046',cursor='pointer')+'</sc-if><sc-if value="{{broken}}" hint-placeholder-val="{{false}}">'+tag('button','Apply the fix',attrs=' type="button" onClick="{{fixIt}}"',font='600 12px Inter, sans-serif',padding='6px 10px',min_height='32px',border_radius='6px',background='#6FD3E6',color='#0B0D0F',border='1px solid #6FD3E6',cursor='pointer')+'</sc-if>'+extra,gap='6px')
    return row(tag('span',f'<sc-if value="{{{{notBroken}}}}" hint-placeholder-val="{{{{true}}}}">{ok}</sc-if><sc-if value="{{{{broken}}}}" hint-placeholder-val="{{{{false}}}}">{bad}</sc-if>',font='12px '+MONO)+btns,justify_content='space-between',gap='10px',flex_wrap='wrap',padding='8px 12px',background='#14171A',border_top='1px solid #2C3035')
def diag_panel():
    return '<sc-if value="{{broken}}" hint-placeholder-val="{{false}}">'+div(col(row(chip('T13','danger',background='#3A1714',color='#FF8A80')+tag('span','A name nothing declares',font='600 13px Inter, sans-serif',color='#FFD7D5'),gap='8px')+tag('pre',E('Home.wf:6:20  `cout` — nothing declares it\n  = help: did you mean `count`?'),margin='0',font='12.5px/1.6 '+MONO,color='#FFD7D5',white_space='pre-wrap')+row(a('Read T13 →','Unified-Error.dc.html',color='#8FBDE7',font_weight='600',font_size='13px')+tag('span','· the same page `wf explain T13` prints',font_size='12px',color='#B3BAC0'),gap='8px',flex_wrap='wrap'),gap='8px'),padding='12px 14px',background='#2A1215',border_top='1px solid #5A2420')+'</sc-if>'
def live_editor2(bid='tour'):
    ed='<sc-if value="{{notBroken}}" hint-placeholder-val="{{true}}">'+editor_body(TOUR_SRC)+'</sc-if><sc-if value="{{broken}}" hint-placeholder-val="{{false}}">'+editor_body(BROKEN_SRC,bad_word='cout')+'</sc-if>'
    prev=tag('div',div(counter_preview(),padding='28px'),attrs=' class="grid"',background_color='var(--surface)',height='100%')
    prev_wrap='<sc-if value="{{notBroken}}" hint-placeholder-val="{{true}}">'+prev+'</sc-if><sc-if value="{{broken}}" hint-placeholder-val="{{false}}">'+div(prev,opacity='.45',filter='grayscale(1)',height='100%')+'</sc-if>'
    split=col(div(div(ed,flex='1 1 360px',min_width='0',background='var(--term)')+div(prev_wrap,flex='1 1 260px',min_width='0',border_left='1px solid #2C3035'),display='flex',flex_wrap='wrap')+diag_panel()+status_bar(),gap='0')
    html_out='''<main id="wf-main">
  <h1 class="wf-heading wf-heading--h1">Hello, WebFluent</h1>
  <p class="wf-text">Clicked 3 times</p>
  <button class="wf-button wf-button--primary" type="button">+1</button>
</main>'''
    js_out='''const count = WF.signal(0);
WF.page("Home", (root) => {
  WF.el("p", { class: "wf-text" }, () => `Clicked ${count()} times`);
  // on click → count.set(count() + 1)
});'''
    return codeblock(bid,[('wf','Home.wf',split),('html','HTML',pre(html_out,'html')),('js','JS',pre(js_out,'js'))],extra=a(icon('play',13)+'Open in Playground','Unified-Playground.dc.html',color='#E6E9EC',font='600 12px Inter, sans-serif',display='inline-flex',gap='6px',align_items='center',padding='6px 10px',border='1px solid #3A4046',border_radius='6px',min_height='32px'))
pages.live_editor=live_editor2
# --- richer output cards on home
IMG={'pdf':'/_blob/53fd25e2de6377a851979dc8e8acb3f3','slides':'/_blob/be466731cee95a92b721797be7def6ec','report':'/_blob/713c5d0836ced456f1991b7ec979d9f0'}
def thumb(kind):
    box=dict(height='120px',border_radius='8px',overflow='hidden',border='1px solid var(--line)',background='var(--sunken)',display='flex',align_items='center',justify_content='center')
    if kind in ('pdf','slides'):
        alt={'pdf':'A WebFluent invoice rendered as a PDF page','slides':'A slide from a WebFluent deck'}[kind]
        return div(f'<img src="{IMG[kind]}" alt="{alt}" style="{st(width="100%",height="100%",object_fit="cover",object_position="top")}">',**box)
    if kind=='web': inner=col(row(''.join(div('',width='6px',height='6px',border_radius='3px',background='var(--line)') for _ in range(3)),gap='4px')+div('',height='10px',width='70%',background='var(--ink)',border_radius='3px',opacity='.8')+div('',height='6px',width='90%',background='var(--line)',border_radius='3px')+div('',height='6px',width='60%',background='var(--line)',border_radius='3px')+div('',height='16px',width='40%',background='var(--brand)',border_radius='4px'),gap='7px',width='70%',padding='12px',background='var(--surface)',border_radius='6px',border='1px solid var(--line)')
    elif kind=='app': inner=row(col(''.join(div('',height='6px',width='100%',background='var(--line)',border_radius='3px') for _ in range(4)),gap='6px',width='26%')+col(div('',height='8px',width='60%',background='var(--ink)',opacity='.8',border_radius='3px')+''.join(row(div('',width='10px',height='10px',border_radius='5px',background='var(--accent-subtle)')+div('',height='6px',flex='1',background='var(--line)',border_radius='3px'),gap='6px') for _ in range(3)),gap='7px',flex='1'),gap='10px',width='76%',padding='12px',background='var(--surface)',border_radius='6px',border='1px solid var(--line)',align_items='flex-start')
    elif kind=='el': inner=tag('code',E('<price-tag amount="12.99">'),font='13px '+MONO,color='var(--accent-ink)',padding='8px 10px',background='var(--surface)',border='1px solid var(--line)',border_radius='6px')
    else: inner=div(div('',height='64%',width='100%',background='var(--surface)',border_radius='6px',border='1px solid var(--line)'),width='64px',height='96px',border_radius='12px',background='var(--ink)',padding='6px',display='flex',align_items='center')
    return div(inner,**box)
def home2(phone=False):
    body=pages.home()
    return body
_old_home=pages.home
def out_cards():
    outs=[('A website','Static, pre-rendered, fast first paint','wf init -t static','web'),('A web app','Routes, stores, live data','wf init -t spa','app'),('A PDF','Reports, invoices, CVs — no browser needed','wf init -t pdf','pdf'),('A slide deck','One slide, one page','wf init -t slides','slides'),('Custom elements','Drop into React, Rails, WordPress','output_type: elements','el'),('A desktop or phone app','The same project in Tauri or Electron','wf build → Tauri','phone')]
    return ''.join(a(col(thumb(k)+row(txt(t,'h3',font_size='17px',font_weight='700')+icon('arrow',16),justify_content='space-between')+txt(s,font_size='14px',color='var(--muted)',line_height='1.5')+tag('code',E(c),font='12px '+MONO,color='var(--accent-ink)',margin_top='auto'),gap='10px',height='100%'),'Unified-Learn.dc.html',display='block',padding='14px 14px 18px',border='1px solid var(--line)',border_radius='12px',background='var(--surface)') for t,s,c,k in outs)
import re
def home_refined():
    b=_old_home()
    # swap the plain output grid for the illustrated one
    start=b.index('<div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr))')
    end=b.index('</section>',start)
    b=b[:start]+div(out_cards(),display='grid',grid_template_columns='repeat(auto-fill, minmax(260px, 1fr))',gap='14px')+'</div>'+b[end:]
    return b
pages.home=home_refined
# --- Playground
def playground():
    files=['src/App.wf','src/pages/Home.wf','src/stores/Counter.wf','src/translations/en.json','tests/home.wf','webfluent.app.json']
    tree=tag('nav',col(label('Files')+''.join(a(f,'#',font='12.5px '+MONO,display='block',padding='5px 8px',border_radius='5px',color='#E6E9EC' if 'Home' in f else '#959BA1',background='#22272E' if 'Home' in f else 'transparent') for f in files)+div('',height='1px',background='#2C3035',margin='8px 0')+label('Examples')+''.join(a(x,'#',font_size='13px',display='block',padding='5px 8px',color='#B3BAC0') for x in ['Counter','Todo list with a store','Invoice (PDF)','Slide deck','Form with validation','Arabic, right to left']),gap='4px'),attrs=' aria-label="Project files"',flex='0 1 220px',min_width='190px',padding='14px 10px',background='#14171A',border_right='1px solid #2C3035')
    ed='<sc-if value="{{notBroken}}" hint-placeholder-val="{{true}}">'+editor_body(TOUR_SRC)+'</sc-if><sc-if value="{{broken}}" hint-placeholder-val="{{false}}">'+editor_body(BROKEN_SRC,bad_word='cout')+'</sc-if>'
    edcol=col(row(row(tag('span','Home.wf',font='12.5px '+MONO,color='#E6E9EC',padding='9px 12px',border_bottom='2px solid #6FD3E6')+tag('span','Counter.wf',font='12.5px '+MONO,color='#959BA1',padding='9px 12px'),gap='0'),gap='0',border_bottom='1px solid #2C3035',background='#14171A')+div(ed,flex='1',background='var(--term)')+diag_panel()+status_bar(),gap='0',flex='999 1 420px',min_width='0',background='var(--term)')
    dev=row('<sc-if value="{{isDesktop}}" hint-placeholder-val="{{true}}">'+chip('Desktop','brand')+'</sc-if><sc-if value="{{isPhone}}" hint-placeholder-val="{{false}}">'+tag('button','Desktop',attrs=' type="button" onClick="{{toDesktop}}"',font='600 12px Inter, sans-serif',padding='4px 10px',border_radius='999px',border='1px solid var(--line)',background='transparent',color='var(--muted)',cursor='pointer',min_height='28px')+'</sc-if>'
        +'<sc-if value="{{isPhone}}" hint-placeholder-val="{{false}}">'+chip('Phone','brand')+'</sc-if><sc-if value="{{isDesktop}}" hint-placeholder-val="{{true}}">'+tag('button','Phone',attrs=' type="button" onClick="{{toPhone}}"',font='600 12px Inter, sans-serif',padding='4px 10px',border_radius='999px',border='1px solid var(--line)',background='transparent',color='var(--muted)',cursor='pointer',min_height='28px')+'</sc-if>',gap='4px')
    page_prev=div(counter_preview(),padding='28px',background='var(--surface)',border='1px solid var(--line)',border_radius='10px',min_height='300px')
    framed='<sc-if value="{{isDesktop}}" hint-placeholder-val="{{true}}">'+page_prev+'</sc-if><sc-if value="{{isPhone}}" hint-placeholder-val="{{false}}">'+div(div(counter_preview(),padding='22px',background='var(--surface)',border_radius='22px',min_height='420px'),width='260px',margin='0 auto',padding='10px',background='var(--ink)',border_radius='30px')+'</sc-if>'
    prevpane=col(row(row(chip('Preview','brand')+chip('HTML')+chip('CSS')+chip('JS')+chip('PDF')+note(9),gap='4px',flex_wrap='wrap')+dev,justify_content='space-between',flex_wrap='wrap',gap='8px')
        +'<sc-if value="{{notBroken}}" hint-placeholder-val="{{true}}">'+framed+'</sc-if><sc-if value="{{broken}}" hint-placeholder-val="{{false}}">'+div(framed,opacity='.45',filter='grayscale(1)')+'</sc-if>'
        +card(col(row(label('Tests')+chip(icon('check',11)+'2 passed','accent'),justify_content='space-between')+tag('code','test "counts clicks" { click "+1"  expect "Clicked 4 times" }',font='12px '+MONO,color='var(--muted)',white_space='normal'),gap='8px'),padding='12px'),gap='12px',flex='1 1 340px',min_width='0',padding='16px',background='var(--bg)')
    toolbar=row(row(txt('counter',font='600 14px '+MONO)+chip('unsaved','plain')+note(2,text='the compiler runs here, in WebAssembly'),gap='8px',flex_wrap='wrap')
        +row(f'<label style="{st(display="inline-flex",align_items="center",gap="6px",font_size="12px",color="var(--muted)")}">Output<select aria-label="Output" style="{st(font="600 13px Inter, sans-serif",padding="6px 8px",min_height="36px",border="1px solid var(--line)",border_radius="6px",background="var(--surface)",color="var(--ink)")}"><option>Website</option><option>PDF</option><option>Slides</option></select></label>'+note(3)
             +btn('Share link',font_size='13px',min_height='36px',padding='6px 12px')+btn('Download project',font_size='13px',min_height='36px',padding='6px 12px')+abtn('Learn this'+icon('arrow',14),'Unified-Learn.dc.html','primary',font_size='13px',min_height='36px',padding='6px 12px'),gap='8px',flex_wrap='wrap'),justify_content='space-between',gap='12px',flex_wrap='wrap',padding='10px 16px',border_bottom='1px solid var(--line)',background='var(--surface)')
    work=div(tree+edcol+prevpane,display='flex',flex_wrap='wrap',align_items='stretch',min_height='640px')
    return header('Playground')+toolbar+work+div(row(note(6,text='errors link to their page')+note(10,text='Share makes a link an agent can read too')+txt('The playground runs the same compiler as <code>wf build</code>, compiled to WebAssembly. Nothing is sent to a server.',font_size='13px',color='var(--muted)'),gap='10px',flex_wrap='wrap'),padding='12px 16px',border_top='1px solid var(--line)',background='var(--surface)')

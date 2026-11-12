from ui import *
def main(c,maxw='1360px',**k): d=dict(max_width=maxw,margin='0 auto',padding='0 24px'); d.update(k); return tag('main',c,**d)
def section(h,sub,body,n=None,**k):
    return tag('section',col(col(row(txt(h,'h2',font_size='28px',font_weight='800',letter_spacing='-.015em')+(note(*n) if n else ''),flex_wrap='wrap')+(txt(sub,font_size='16px',color='var(--muted)',line_height='1.55',max_width='680px') if sub else ''),gap='8px')+body,gap='24px'),margin_top='72px',**k)
TOUR_SRC='''page Home(path: "/", title: "Hello") {
    state count = 0

    Heading("Hello, WebFluent").h1
    Text("Clicked {count} times")
    Button("+1").primary {
        on click { count = count + 1 }
    }
}'''
def counter_preview(): return col(txt('Hello, WebFluent','h1',font_size='28px',font_weight='800')+txt('Clicked {{count}} times',font_size='15px')+row(tag('button','+1',attrs=' type="button" onClick="{{inc}}"',font='600 14px Inter, sans-serif',padding='10px 18px',min_height='44px',border_radius='6px',background='var(--brand)',color='#fff',border='1px solid var(--brand)',cursor='pointer')+tag('button','Reset',attrs=' type="button" onClick="{{reset}}"',font='600 14px Inter, sans-serif',padding='10px 14px',min_height='44px',border_radius='6px',background='var(--surface)',color='var(--ink)',border='1px solid var(--line)',cursor='pointer'),gap='8px'),gap='12px',align_items='flex-start')
def live_editor(bid='tour'):
    src=pre(TOUR_SRC,padding='18px 20px')
    prev=div(counter_preview(),padding='28px',background='var(--surface)',min_height='100%')
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
    split=div(div(src,flex='1 1 340px',min_width='0',background='var(--term)')+div(prev,flex='1 1 280px',min_width='0',border_left='1px solid #2C3035'),display='flex',flex_wrap='wrap')
    return codeblock(bid,[('wf','Home.wf + Preview',split),('html','HTML',pre(html_out,'html')),('js','JS',pre(js_out,'js'))],extra=chip(icon('check',12)+'0 problems',tone='accent',background='transparent',border='1px solid #2C3035',color='#6FD3E6'))
# ===================== HOME =====================
def home(phone=False):
    hero=wrap(col(note(3,text='the first question is what you are making')+txt('Websites · apps · PDFs · slides',font='600 13px '+MONO,color='var(--accent-ink)')
        +txt('One language. Every page it makes is checked.','h1',font_size='clamp(36px, 5vw, 56px)',line_height='1.04',font_weight='800',letter_spacing='-.03em')
        +txt('WebFluent compiles one typed file to a static site, an app, a PDF or a slide deck — and every example in these docs is compiled by the release it documents.',font_size='18px',line_height='1.6',color='var(--muted)',max_width='560px')
        +row(div(tag('span','curl -fsSL https://webfluent.dev/install.sh | sh',font='13px '+MONO,white_space='nowrap',overflow='hidden',text_overflow='ellipsis'),flex='1',min_width='0')+btn(icon('copy',14),kind='ghost',min_height='36px',padding='6px 8px',attrs=' aria-label="Copy install command"'),padding='6px 6px 6px 14px',border='1px solid var(--line)',border_radius='8px',background='var(--surface)',max_width='520px')
        +wrap(abtn('Start the tour'+icon('arrow',15),'Unified-Learn.dc.html','primary')+abtn(icon('book',15)+'Read the book','Unified-Learn.dc.html')+a('Coming from React?','Unified-ComingFrom.dc.html',font_weight='600',color='var(--brand-ink)',padding='10px 6px'),gap='10px'),gap='20px',flex='1 1 460px',min_width='0')
        +col(note(2,text='edit it — the real page redraws')+live_editor(),gap='10px',flex='1 1 480px',min_width='0'),gap='48px',align_items='flex-start',margin_top='56px')
    outs=[('A website','Static, pre-rendered, fast first paint','wf init -t static','Website'),('A web app','Routes, stores, live data','wf init -t spa','Web app'),('A PDF','Reports, invoices, CVs — no browser needed','wf init -t pdf','PDF'),('A slide deck','One slide, one page','wf init -t slides','Slides'),('Custom elements','Drop into React, Rails, WordPress','output_type: elements','Elements'),('A desktop or phone app','The same project in Tauri or Electron','wf build → Tauri','Desktop')]
    oc=''.join(a(col(row(txt(t,'h3',font_size='17px',font_weight='700')+icon('arrow',16),justify_content='space-between')+txt(s,font_size='14px',color='var(--muted)',line_height='1.5')+tag('code',E(c),font='12px '+MONO,color='var(--accent-ink)',margin_top='auto'),gap='8px',height='100%'),'Unified-Learn.dc.html',display='block',padding='20px',border='1px solid var(--line)',border_radius='10px',background='var(--surface)',min_height='150px') for t,s,c,_ in outs)
    outsec=section('What are you making?','Pick one and the guide, the recipes and the reference filter to what applies. The switch in the header changes it anywhere.',div(oc,display='grid',grid_template_columns='repeat(auto-fill, minmax(260px, 1fr))',gap='14px'),n=(3,))
    froms=[('React','useState → state'),('Vue','computed → derived'),('Svelte','{#each} → for … by'),('HTML & CSS','class → flags and style { }'),('LaTeX / Typst','\\section → Heading, PDF out')]
    fc=''.join(a(col(txt(f,font_weight='700',font_size='15px')+tag('code',E(m),font='12.5px '+MONO,color='var(--muted)'),gap='6px'),'Unified-ComingFrom.dc.html',display='block',padding='16px',border='1px solid var(--line)',border_radius='10px',background='var(--surface)',flex='1 1 200px') for f,m in froms)
    fromsec=section('Coming from somewhere?','A one-page cheat sheet for what you already know, and the habits that will trip you.',wrap(fc,gap='12px',align_items='stretch'),n=(4,))
    rooms=[('Learn','Tutorial','The tour, then the book: one linear path, 3 lessons in the browser, then 22 chapters.',['1 · Getting started','4 · State and reactivity','About: the security model'],'Unified-Learn.dc.html',(2,7)),
           ('Recipes','How-to','Complete, runnable answers to “how do I…”, each linked to what it uses.',['Fetch and show a list','Validate a sign-up form','Ship a PDF invoice'],'Unified-Recipe.dc.html',(8,)),
           ('Reference','Reference','77 components, the CLI, config and runtime — generated from the compiler, live.',['Button','Grid','wf build'],'Unified-Reference.dc.html',(5,)),
           ('Errors','Error','Every diagnostic has a page: the broken code, the real message, the fix.',['T05 · no such field','E002 · syntax','A05 · button with no text'],'Unified-Error.dc.html',(6,))]
    rc=''.join(card(col(row(txt(n,'h3',font_size='20px',font_weight='800')+mode(m),justify_content='space-between')+txt(d,font_size='14px',color='var(--muted)',line_height='1.55')+col(''.join(a(x,href,color='var(--brand-ink)',font_weight='600') for x in xs),gap='8px')+note(*o),gap='12px'),padding='22px',flex='1 1 260px') for n,m,d,xs,href,o in rooms)
    roomsec=section('Find your way','Four kinds of page, and each page says which it is — so you know whether to study it or act on it.',wrap(rc,gap='14px',align_items='stretch'),n=(1,))
    under=card(wrap(col(label('See through it')+txt('Every example shows what it compiles to','h3',font_size='22px',font_weight='800')+txt('Switch any code block to HTML, CSS, JS or the PDF page. Hover a token to see where it came from. Nothing is hidden.',font_size='15px',color='var(--muted)',line_height='1.55')+note(9),gap='10px',flex='1 1 360px')
        +col(label('For your agent')+txt('The same docs, as clean Markdown','h3',font_size='22px',font_weight='800')+txt('Copy any page, read llms.txt, or point an agent at the MCP endpoint. It reads the same verified pages you do.',font_size='15px',color='var(--muted)',line_height='1.55')+wrap(chip('llms.txt')+chip('Copy page')+chip('wf context')+chip('MCP'),gap='6px')+note(10),gap='10px',flex='1 1 360px'),gap='40px',align_items='flex-start'),padding='28px')
    body=header()+main(hero+outsec+fromsec+roomsec+tag('section',under,margin_top='72px'))+footer()
    return body
# ===================== LEARN =====================
def learn():
    chs=[('1','Getting started',True,'done'),('2','A first page',True,'done'),('3','Elements and flags',True,'done'),('4','State and reactivity',False,'now'),('5','Components',False,''),('6','Stores',False,''),('7','Data and resources',False,''),('8','Styling and themes',False,''),('9','Forms',False,''),('10','PDF and slides',False,''),('11','Shipping',False,'')]
    items=''
    for n,t,tour,stt in chs:
        on=stt=='now'
        items+=a(row(tag('span',icon('check',13) if stt=='done' else n,width='22px',height='22px',display='inline-flex',align_items='center',justify_content='center',border_radius='11px',font='600 11px '+MONO,background='var(--ok-subtle)' if stt=='done' else ('var(--brand)' if on else 'var(--sunken)'),color='var(--ok)' if stt=='done' else ('#fff' if on else 'var(--muted)'),flex='none')+tag('span',t,flex='1')+(chip(icon('play',10)+'tour','accent',padding='1px 7px',font_size='11px') if tour else ''),gap='10px'),'#',display='block',padding='7px 8px',border_radius='6px',background='var(--brand-subtle)' if on else 'transparent',color='var(--ink)',font_weight='600' if on else '400')
    abouts=''.join(a(x,color='var(--muted)',padding='5px 8px',display='block',font_size='13.5px') for x in ['About reactivity','About the security model','About how PDF layout works','Under the hood: what a build ships'])
    side=tag('aside',col(col(row(label('The book')+txt('4 of 11',font_size='12px',color='var(--muted)'),justify_content='space-between')+div(div('',width='31%',height='100%',background='var(--brand)',border_radius='3px'),height='6px',background='var(--line)',border_radius='3px')+note(7,text='one numbered spine'),gap='8px')+col(items,gap='2px')+note(2,text='lessons 1–3 run in the browser')+col(label('About')+abouts,gap='4px')+note(1,9,text='explanation lives beside the path'),gap='16px'),attrs=' aria-label="Book contents"',flex='0 1 260px',min_width='220px',padding='28px 0')
    prose=lambda t: txt(t,font_size='17px',line_height='1.75',max_width='680px')
    quiz=card(col(row(row(icon('check',16)+txt('Check your understanding',font_weight='700',font_size='15px'),gap='8px')+note(7,text='answers tell us which paragraph to rewrite'),justify_content='space-between',flex_wrap='wrap')
        +txt('A page has <code>state n = 0</code> and <code>derived d = n * 2</code>, and shows <code>Text("{d}")</code>. You set <code>n = 5</code>. What redraws?',font_size='15px',line_height='1.6')
        +col(''.join(f'<label style="{st(display="flex",gap="10px",align_items="center",padding="12px 14px",border="1px solid var(--line)",border_radius="8px",font_size="14px",cursor="pointer",min_height="44px")}"><input type="radio" name="quiz" onChange="{{{{pick{i}}}}}"> {o}</label>' for i,o in enumerate(['The whole page draws again','Only the text that reads d','Nothing, until you call render()'])),gap='8px')
        +row(tag('button','Check answer',attrs=' type="button" onClick="{{check}}"',font='600 14px Inter, sans-serif',padding='10px 16px',min_height='44px',border_radius='6px',background='var(--brand)',color='#fff',border='1px solid var(--brand)',cursor='pointer')
             +'<sc-if value="{{quizRight}}" hint-placeholder-val="{{false}}">'+chip(icon('check',12)+'Right — only what reads d redraws, and it reads n through it.','accent')+'</sc-if>'
             +'<sc-if value="{{quizWrong}}" hint-placeholder-val="{{false}}">'+chip(icon('x',12)+'Not quite — re-read “What redraws” above.','danger')+'</sc-if>',gap='12px',flex_wrap='wrap'),gap='14px'),padding='22px',background='var(--sunken)')
    react=card(row(chip('React','plain')+txt('Coming from React? <code>useState</code> is <code>state</code>, and there is no setter: assign to it. <a href="Unified-ComingFrom.dc.html">The cheat sheet →</a>',font_size='14px',line_height='1.55')+note(4),gap='12px',flex_wrap='wrap'),padding='14px 16px')
    content=tag('article',col(row(wrap(mode('Tutorial')+chip('Chapter 4')+chip('Web · App · PDF','plain')+note(3,text='applies to every output'),gap='8px')+copymenu()+note(10),justify_content='space-between',flex_wrap='wrap')
        +txt('State and reactivity','h1',font_size='clamp(32px, 4vw, 44px)',font_weight='800',letter_spacing='-.025em',line_height='1.1')
        +prose('This is the chapter most readers find hardest, so it goes slowly. A value you declare with <code>state</code> is read by whatever shows it — and only what reads it draws again when it changes.')
        +col(row(txt('Try it','h2',font_size='20px',font_weight='800')+note(2,9,text='edit, run, and see the compiled output'),flex_wrap='wrap')+live_editor('learn'),gap='12px')
        +txt('What redraws','h2',font_size='24px',font_weight='800',margin_top='12px')
        +prose('Nothing re-runs the whole page. The compiler sees that <code>Text("Clicked {count} times")</code> reads <code>count</code>, and ties that one text node to it. Click <b>+1</b> above: only that line changes.')
        +react+quiz
        +row(abtn(icon('back',15)+'3 · Elements and flags','#')+abtn('5 · Components'+icon('arrow',15),'#','primary'),justify_content='space-between',flex_wrap='wrap',gap='10px',margin_top='12px'),gap='22px'),flex='999 1 560px',min_width='0',padding='28px 0')
    rail=tag('aside',col(label('On this page')+''.join(a(x,color='var(--muted)',font_size='13.5px') for x in ['Try it','What redraws','derived values','Check your understanding'])
        +div('',height='1px',background='var(--line)',margin='6px 0')+label('Errors you may meet here')+''.join(a(row(chip(c,'danger')+tag('span',t,font_size='13px',color='var(--muted)'),gap='8px'),'Unified-Error.dc.html') for c,t in [('X01','assigning a derived'),('X06','state that never ticks'),('T17','await outside an action')])+note(6),gap='10px'),attrs=' aria-label="On this page"',flex='0 1 220px',min_width='200px',padding='32px 0')
    return header('Learn')+main(div(side+content+rail,display='flex',flex_wrap='wrap',gap='40px',align_items='flex-start'))+footer()
# ===================== RECIPE =====================
def recipe():
    src='''type User { id: String, name: String }

page Team(path: "/team", title: "Team") {
    resource users: [User] = fetch("/api/users", on: .focus)

    match users {
        loading { Skeleton(height: "20px") }
        error(e) { Alert(e.message).danger }   // a first load that failed
        ready(list) {
            for u in list by u.id {             // keyed: rows keep their state
                Text(u.name)
            }
        }
    }
}'''
    prev=preview_panel(col(''.join(row(div(txt(n[0],font_weight='700',font_size='13px'),width='32px',height='32px',border_radius='16px',background='var(--brand-subtle)',color='var(--brand-ink)',display='flex',align_items='center',justify_content='center')+txt(n,font_size='15px'),gap='10px') for n in ['Ada Lovelace','Grace Hopper','Margaret Hamilton']),gap='10px',align_items='flex-start',width='100%'),justify_content='flex-start')
    html_out='''<main id="wf-main">
  <p class="wf-text">Ada Lovelace</p>
  <p class="wf-text">Grace Hopper</p>
  <p class="wf-text">Margaret Hamilton</p>
</main>'''
    js_out='''const users = WF.resource("/api/users", { on: "focus" });
WF.match(root, () => users.state, null, {
  loading: () => WF.el("div", { class: "wf-skeleton" }),
  ready: (list) => WF.each(root, list, row, { key: (u) => u.id }),
});'''
    cb=codeblock('rec',[('wf','Team.wf',pre(src)),('preview','Preview',prev),('html','HTML',pre(html_out,'html')),('js','JS',pre(js_out,'js'))])
    used=wrap(''.join(a(chip(x,'brand'),'Unified-Reference.dc.html') for x in ['resource','match','for … by','Skeleton','Alert','Text']),gap='6px')
    content=tag('article',col(txt('<a href="#">Recipes</a> / Fetch',font_size='13px',color='var(--muted)')
        +row(wrap(mode('How-to')+chip(icon('check',11)+'Web','accent')+chip(icon('check',11)+'App','accent')+chip(icon('x',11)+'PDF','off')+note(3,text='where it works'),gap='6px')+copymenu(),justify_content='space-between',flex_wrap='wrap')
        +txt('How to fetch and show a list','h1',font_size='clamp(30px, 4vw, 42px)',font_weight='800',letter_spacing='-.025em',line_height='1.1')
        +txt('Ask a server for a list, show a placeholder while it comes, and draw each item. Done in 14 lines.',font_size='17px',line_height='1.6',color='var(--muted)',max_width='680px')
        +row(note(8,text='complete, runnable, comments carry the rules')+note(9),flex_wrap='wrap',gap='8px')+cb
        +txt("What's happening",'h2',font_size='22px',font_weight='800')
        +txt('<code>resource</code> starts the request when the page draws and asks again when the tab regains focus. <code>match</code> draws one arm for its state: the skeleton while loading, the list when ready.',font_size='16px',line_height='1.7',max_width='680px')
        +txt('A refresh keeps the list on screen and redraws in place, so a row keeps its focus. A refresh that fails keeps the old data and sets <code>users.error</code>.',font_size='16px',line_height='1.7',max_width='680px')
        +card(col(row(label('Used here')+note(5,text='every keyword links to its reference'),justify_content='space-between',flex_wrap='wrap')+used,gap='10px'),padding='16px')
        +txt('Variations','h2',font_size='22px',font_weight='800')
        +col(''.join(a(row(txt(t,font_weight='600')+icon('arrow',15),justify_content='space-between'),'#',display='block',padding='14px 16px',border='1px solid var(--line)',border_radius='8px',background='var(--surface)') for t in ['…with pagination (paginate: .page)','…refreshed every 30 seconds','…from a typed api with retries']),gap='8px'),gap='20px'),flex='999 1 560px',min_width='0',padding='28px 0')
    side=tag('aside',col(label('Recipes')+''.join(a(x,padding='7px 8px',display='block',border_radius='6px',background='var(--brand-subtle)' if x.startswith('Fetch and show') else 'transparent',font_weight='600' if x.startswith('Fetch and show') else '400') for x in ['Fetch and show a list','Refresh on a timer','Validate a sign-up form','Add dark mode','Ship a PDF invoice','Translate to Arabic','Test a click-through'])+note(8,text='grouped by verb, not by feature'),gap='4px'),attrs=' aria-label="Recipes"',flex='0 1 240px',min_width='210px',padding='28px 0')
    rail=tag('aside',col(label('Seen in the showcase')+card(col(div('',height='110px',background='var(--sunken)',border_radius='6px',border='1px dashed var(--line)')+txt('Halyard — platform report',font_weight='700',font_size='14px')+txt('Uses this recipe for its deploy list.',font_size='13px',color='var(--muted)'),gap='8px'),padding='12px')+note(8,text='showcase apps are made of recipes'),gap='10px'),flex='0 1 240px',min_width='200px',padding='32px 0')
    return header('Recipes')+main(div(side+content+rail,display='flex',flex_wrap='wrap',gap='40px',align_items='flex-start'))+footer()
# ===================== REFERENCE =====================
def reference():
    cats=[('Actions',['Button','IconButton','ButtonGroup','Dropdown']),('Layout',['Container','Row','Stack','Grid']),('Form & input',['Input','Select','Checkbox','Switch']),('Feedback',['Alert','Toast','Modal'])]
    side=f'<label style="{st(display="flex",align_items="center",gap="8px",padding="0 10px",min_height="40px",border="1px solid var(--line)",border_radius="8px",background="var(--sunken)",color="var(--muted)")}">{icon("search",14)}<input type="search" placeholder="Filter 77 built-ins" aria-label="Filter components" style="{st(border="0",background="transparent",font="13px Inter, sans-serif",color="var(--ink)",flex="1",min_width="0",outline="none")}"></label>'
    for g,xs in cats:
        side+=label(g)+''.join(a(x,padding='6px 8px',display='block',border_radius='6px',background='var(--brand-subtle)' if x=='Button' else 'transparent',font_weight='600' if x=='Button' else '400',font_size='14px') for x in xs)
    side=tag('aside',col(side+note(5,text='generated from wf registry'),gap='6px'),attrs=' aria-label="Components"',flex='0 1 240px',min_width='210px',padding='28px 0')
    sel=lambda lab,opts,h: f'<label style="{st(display="flex",flex_direction="column",gap="6px",font="600 12px Inter, sans-serif",color="var(--muted)")}">{lab}<select onChange="{{{{{h}}}}}" style="{st(font="14px Inter, sans-serif",padding="8px 10px",min_height="40px",border="1px solid var(--line)",border_radius="6px",background="var(--surface)",color="var(--ink)")}">'+''.join(f'<option value="{o}">.{o}</option>' for o in opts)+'</select></label>'
    controls=col(row(label('Controls')+note(5),justify_content='space-between',flex_wrap='wrap')+sel('tone',['primary','secondary','danger','outlined'],'setTone')+sel('size',['md','sm','lg'],'setSize')+f'<label style="{st(display="flex",align_items="center",gap="10px",font_size="14px",min_height="40px")}"><input type="checkbox" onChange="{{{{togglePill}}}}"> .pill</label>',gap='12px',flex='0 1 220px',min_width='200px',padding='18px',border_left='1px solid var(--line)')
    live=div(div(tag('button','Save',attrs=' type="button" style="{{prevStyle}}"'),flex='1 1 300px',display='flex',align_items='center',justify_content='center',min_height='200px',background='var(--sunken)',background_image='radial-gradient(var(--line) 1px, transparent 1px)',background_size='16px 16px')+controls,display='flex',flex_wrap='wrap',border='1px solid var(--line)',border_radius='10px',overflow='hidden',background='var(--surface)')
    snippet=div(row(tag('code','{{snippet}}',font='14px '+MONO,color='#E6E9EC')+btn(icon('copy',14)+'Copy',kind='ghost',color='#E6E9EC',min_height='36px',font_size='12px',padding='6px 10px'),justify_content='space-between',gap='10px'),background='var(--term)',padding='10px 10px 10px 18px',border_radius='10px')
    html_out='''<button class="wf-button wf-button--primary" type="button">
  Save
</button>'''
    css_out='''.wf-button--primary {
  background: var(--color-primary);
  color: var(--color-surface);
  border-color: var(--color-primary);
}'''
    cb=codeblock('ref',[('wf','.wf',pre('Button("Save").primary {\n    on click { save() }\n}')),('html','HTML',pre(html_out,'html')),('css','CSS',pre(css_out,'css'))],title='Usage')
    props=[('label','String','—','What the button says (positional)'),('tone','Tone','.primary','.primary .secondary .success .danger .warning .info'),('size','Size','.md','.sm .md .lg'),('outlined','Bool','false','A border, no fill'),('pill','Bool','false','Fully rounded'),('full','Bool','false','Fills its row'),('type','ButtonType','.button','.submit inside a Form')]
    ptab=f'<div style="{st(overflow_x="auto",border="1px solid var(--line)",border_radius="10px")}"><table style="{st(width="100%",border_collapse="collapse",font_size="14px",background="var(--surface)")}"><thead><tr>'+''.join(f'<th scope="col" style="{st(text_align="left",padding="10px 14px",font="700 12px Inter, sans-serif",color="var(--muted)",border_bottom="1px solid var(--line)",background="var(--sunken)")}">{h}</th>' for h in ['Prop','Type','Default','What it does'])+'</tr></thead><tbody>'+''.join('<tr>'+''.join(f'<td style="{st(padding="10px 14px",border_bottom="1px solid var(--line)",font=("13px "+MONO) if i<3 else "14px Inter, sans-serif",color="var(--brand-ink)" if i==1 else "var(--ink)")}">{E(v)}</td>' for i,v in enumerate(r))+'</tr>' for r in props)+'</tbody></table></div>'
    content=tag('article',col(row(wrap(mode('Reference')+chip('Since 3.0')+chip(icon('check',11)+'Web','accent')+chip(icon('check',11)+'App','accent')+chip(icon('x',11)+'PDF · E109','off')+note(3),gap='6px')+row(copymenu()+note(10),gap='8px'),justify_content='space-between',flex_wrap='wrap')
        +txt('Button','h1',font_size='42px',font_weight='800',letter_spacing='-.025em')
        +txt('A control that does something when pressed. Its block holds what it shows; <code>on click</code> holds what it does.',font_size='17px',line_height='1.6',color='var(--muted)',max_width='680px')
        +live+snippet
        +row(txt('Usage','h2',font_size='22px',font_weight='800')+note(9,text='see what it compiles to'),flex_wrap='wrap')+cb
        +card(row(icon('arrow',16)+txt('Recipes that use Button: <a href="Unified-Recipe.dc.html">Validate a sign-up form</a> · <a href="#">Disable Save while it runs</a>',font_size='14px')+note(8),gap='10px',flex_wrap='wrap'),padding='14px 16px')
        +txt('Props','h2',font_size='22px',font_weight='800')+ptab
        +txt('Events','h2',font_size='22px',font_weight='800')+txt('Every DOM event, on its <code>&lt;button&gt;</code>: <code>on click</code>, <code>on focus</code>, <code>on key("mod+s")</code>…',font_size='15px',line_height='1.6'),gap='18px'),flex='999 1 560px',min_width='0',padding='28px 0')
    rail=tag('aside',col(label('Composition')+pre('ButtonGroup\n├─ Button\n└─ Button\n\nForm\n└─ Button(type: .submit)','txt',border_radius='8px',font='12.5px/1.6 '+MONO)+note(5,10,text='parts and nesting, for people and agents')
        +label('It can raise')+''.join(a(row(chip(c,'danger')+tag('span',t,font_size='13px',color='var(--muted)'),gap='8px'),'Unified-Error.dc.html') for c,t in [('A05','a button with no text'),('A15','aria-label hides its text'),('E106','onclick: as an attribute')])+note(6),gap='10px'),flex='0 1 240px',min_width='210px',padding='32px 0')
    return header('Reference')+main(div(side+content+rail,display='flex',flex_wrap='wrap',gap='40px',align_items='flex-start'))+footer()
# ===================== ERROR =====================
def error_page():
    fams=[('E','Syntax and structure',24),('T','Types',21),('C','Components',7),('R','Routes',4),('X','State',6),('F','Forms',5),('I','Translations',5),('D','Data and assets',5),('A','Accessibility',17),('S','Search',4),('U','Unused',10),('V','Vocabulary',12)]
    side=tag('aside',col(label('Every diagnostic')+''.join(a(row(chip(f,'danger' if f=='T' else 'plain',min_width='28px',justify_content='center')+tag('span',n,flex='1',font_size='14px')+tag('span',str(c),font='12px '+MONO,color='var(--muted)'),gap='10px'),'#',display='block',padding='6px 8px',border_radius='6px',background='var(--sunken)' if f=='T' else 'transparent') for f,n,c in fams)+note(6,text='grouped by family, as the compiler groups them'),gap='4px'),flex='0 1 250px',min_width='220px',padding='28px 0')
    msg='''error[T05]: `User` has no field `nmae`
 --> src/pages/Profile.wf:3:18
  |
3 |     Heading(user.nmae).h1
  |                  ^^^^
  = help: Its fields are `id`, `name`
  = docs: webfluent.dev/errors/T05'''
    fixsrc='''type User { id: String, name: String }

page Profile(path: "/me", title: "Me") {
    state user = User(id: "1", name: "Ada Lovelace")
    Heading(user.name).h1
}'''
    content=tag('article',col(row(wrap(mode('Error')+chip('T05','danger')+chip('Types')+note(1),gap='6px')+row(copymenu(),gap='8px'),justify_content='space-between',flex_wrap='wrap')
        +txt('A field or method that does not exist','h1',font_size='clamp(28px, 4vw, 40px)',font_weight='800',letter_spacing='-.025em',line_height='1.1')
        +txt('You read a field the record’s type does not declare. In the browser it would quietly be <code>undefined</code>, so the compiler stops here instead.',font_size='17px',line_height='1.6',color='var(--muted)',max_width='680px')
        +row(txt('What the compiler said','h2',font_size='18px',font_weight='800')+note(6,text='the exact message you just saw'),flex_wrap='wrap')
        +div(pre(msg,'txt',background='transparent',color='#FFD7D5'),background='#2A1215',border_radius='10px',overflow='hidden',border='1px solid #5A2420')
        +row(txt('The fix','h2',font_size='18px',font_weight='800')+note(9,2,text='fix and result, side by side'),flex_wrap='wrap')
        +div(div(pre(fixsrc),flex='1 1 360px',min_width='0',background='var(--term)')+div(txt('Ada Lovelace','h1',font_size='30px',font_weight='800'),flex='1 1 220px',padding='28px',display='flex',align_items='center',background='var(--surface)'),display='flex',flex_wrap='wrap',border='1px solid var(--line)',border_radius='10px',overflow='hidden')
        +txt('Why this rule exists','h2',font_size='18px',font_weight='800')
        +txt('A typo in a field name is the most common bug a type checker can catch before a reader sees it. The nearest name is offered as a fix, and your editor applies it.',font_size='16px',line_height='1.7',max_width='680px')
        +card(row(chip('TypeScript','plain')+txt('This is TypeScript’s <i>Property does not exist on type</i>.',font_size='14px')+note(4),gap='10px',flex_wrap='wrap'),padding='14px 16px')
        +wrap(abtn(icon('book',15)+'Learn: Types and records','Unified-Learn.dc.html')+abtn('Related: T04 · T06 · T16','#'),gap='10px'),gap='18px'),flex='999 1 560px',min_width='0',padding='28px 0')
    rail=tag('aside',col(label('In your editor')+card(col(txt('wf explain T05',font='13px '+MONO)+txt('Prints this page in the terminal.',font_size='13px',color='var(--muted)'),gap='6px'),padding='12px')+label('Seen most this month')+''.join(a(row(chip(c,'danger')+tag('span',t,font_size='13px',color='var(--muted)'),gap='8px'),'#') for c,t in [('E002','a syntax error'),('T13','a name nothing declares'),('V01','a flag without its dot')]),gap='10px'),flex='0 1 220px',min_width='200px',padding='32px 0')
    return header('Errors')+main(div(side+content+rail,display='flex',flex_wrap='wrap',gap='40px',align_items='flex-start'))+footer()
# ===================== COMING FROM =====================
def coming():
    pairs=[('A value that changes','const [n, setN] = useState(0)\nsetN(n + 1)','state n = 0\nn = n + 1'),('A value worked out','const total = useMemo(\n  () => sum(items), [items])','derived total = items.sum()'),('A click','<button onClick={add}>Add</button>','Button("Add") { on click { add() } }'),('Show if','{open && <Panel />}','if open { Panel() }'),('A list','{items.map(i =>\n  <Row key={i.id} item={i} />)}','for i in items by i.id { Row(i) }'),('Shared state','const useCart = create(...)','store Cart { state items = [] }'),('Fetching','useEffect(() => { fetch(...) }, [])','resource users = fetch("/api/users")')]
    rows=''.join(div(txt(t,font_weight='700',font_size='14px')+div(tag('pre',E(r),margin='0',font='12.5px/1.6 '+MONO,white_space='pre-wrap',color='var(--ink)'),padding='12px 14px',background='var(--sunken)',border='1px solid var(--line)',border_radius='8px',min_width='0')+div(pre(w,padding='12px 14px',white_space='pre-wrap',border_radius='8px'),min_width='0'),display='grid',grid_template_columns='minmax(120px, 0.6fr) minmax(0, 1fr) minmax(0, 1fr)',gap='12px',align_items='center') for t,r,w in pairs)
    head=div(tag('span','')+label('React')+row(label('WebFluent')+note(4,text='every right-hand line is compiled in CI'),flex_wrap='wrap'),display='grid',grid_template_columns='minmax(120px, 0.6fr) minmax(0, 1fr) minmax(0, 1fr)',gap='12px')
    diff=''.join(row(chip(c,'danger')+txt(t,font_size='14px',line_height='1.5'),gap='10px',align_items='flex-start') for c,t in [('E106','There is no <code>onClick</code> attribute. A handler is a block: <code>on click { }</code>.'),('E002','Arguments are separated by commas; statements by new lines — never a comma between statements.'),('V01','A flag needs its dot after the parentheses: <code>Button("Save").primary</code>.'),('X01','A <code>derived</code> cannot be assigned. Change what it reads instead.')])
    tabs_=row(''.join(a(t,'#',padding='8px 12px',border_radius='6px',font_weight='600',background='var(--brand-subtle)' if t=='React' else 'transparent',color='var(--brand-ink)' if t=='React' else 'var(--muted)') for t in ['React','Vue','Svelte','HTML & CSS','LaTeX / Typst','New to programming']),gap='4px',flex_wrap='wrap')
    content=col(row(wrap(mode('Reference')+chip('Coming from','accent')+note(4),gap='6px')+copymenu(),justify_content='space-between',flex_wrap='wrap',margin_top='28px')
        +txt('WebFluent for React developers','h1',font_size='clamp(30px, 4vw, 44px)',font_weight='800',letter_spacing='-.025em')
        +txt('Your idiom on the left, ours on the right. Five minutes to read — then try the tour.',font_size='17px',color='var(--muted)')
        +tabs_+card(col(head+rows,gap='12px'),padding='20px')
        +card(col(row(txt('Things that will trip you','h2',font_size='20px',font_weight='800')+note(6,text='from the diagnostics switchers hit most'),flex_wrap='wrap')+diff,gap='14px'),padding='22px',background='var(--sunken)')
        +card(row(icon('search',16)+txt('Search knows your words: type <code>useEffect</code>, <code>computed</code> or <code>v-for</code> and land on the WebFluent equivalent.',font_size='14px')+note(4),gap='10px',flex_wrap='wrap'),padding='14px 16px')
        +wrap(abtn('Start the tour'+icon('arrow',15),'Unified-Learn.dc.html','primary')+abtn('The book, chapter 4: State','Unified-Learn.dc.html'),gap='10px'),gap='20px',max_width='1100px')
    return header('Learn')+main(content)+footer()
# ===================== SEARCH =====================
def search_page():
    bg=tag('div',header()+main(col(div('',height='56px')+txt('One language. Every page it makes is checked.','h2',font_size='52px',font_weight='800',letter_spacing='-.03em',max_width='700px')+div('',height='220px',background='var(--surface)',border='1px solid var(--line)',border_radius='10px'))),attrs=' aria-hidden="true" inert=""')
    groups=[('Coming from React',[('useState → state','A value that changes · cheat sheet','Unified-ComingFrom.dc.html','accent')]),('Learn',[('4 · State and reactivity','Tutorial · the book','Unified-Learn.dc.html','brand')]),('Reference',[('Input(bind:)','Two-way binding to a state','Unified-Reference.dc.html','plain'),('persist','State kept across visits','Unified-Reference.dc.html','plain')]),('Errors',[('X06','A state that never ticks','Unified-Error.dc.html','danger')]),('Recipes',[('Keep a form draft between visits','How-to','Unified-Recipe.dc.html','accent')])]
    res=''
    first=True
    for g,items in groups:
        res+=txt(g,font='700 11px Inter, sans-serif',letter_spacing='.08em',text_transform='uppercase',color='var(--muted)',padding='10px 16px 4px')
        for t,s,h,tone in items:
            res+=a(row(chip(icon('arrow',11),tone,padding='4px 6px')+col(txt(t,font_weight='600',font_size='14px')+txt(s,font_size='12.5px',color='var(--muted)'),gap='2px',flex='1')+(tag('kbd','↵',font='11px '+MONO,border='1px solid var(--line)',border_radius='4px',padding='1px 6px') if first else ''),gap='12px'),h,display='block',padding='10px 16px',background='var(--brand-subtle)' if first else 'transparent',border_radius='8px',margin='0 6px')
            first=False
    ask=div(col(row(row(icon('bot',15)+txt('Ask the docs',font_weight='700',font_size='13px'),gap='6px')+row(chip('answers from v5.12 only')+note(10),gap='6px'),justify_content='space-between',flex_wrap='wrap')+txt('“How do I keep a value when the reader comes back?” → Use <code>persist</code> instead of <code>state</code>. <a href="#">Sources: State §persist, Recipe: form drafts</a>',font_size='13px',line_height='1.5'),gap='8px'),padding='14px 16px',border_top='1px solid var(--line)',background='var(--sunken)')
    pal=div(col(row(icon('search',18)+f'<input type="search" value="useState" aria-label="Search the docs" style="{st(border="0",background="transparent",font="500 18px Inter, sans-serif",color="var(--ink)",flex="1",min_width="0",outline="none")}">'+tag('kbd','esc',font='11px '+MONO,border='1px solid var(--line)',border_radius='4px',padding='1px 6px'),gap='12px',padding='16px 18px',border_bottom='1px solid var(--line)')
        +row(chip('All','brand')+chip('Learn')+chip('Recipes')+chip('Reference')+chip('Errors')+note(1,text='scope by kind of page'),gap='6px',padding='10px 16px',flex_wrap='wrap')
        +div(res,padding='4px 0 10px')+ask,gap='0'),width='min(680px, calc(100% - 32px))',background='var(--surface)',border='1px solid var(--line)',border_radius='14px',box_shadow='0 4px 8px rgba(23,25,28,0.06), 0 24px 60px rgba(23,25,28,0.28)',overflow='hidden')
    overlay=div(col(row(note(4,text='foreign words map to ours')+note(6,text='codes jump straight to their page'),gap='8px',flex_wrap='wrap',justify_content='center')+pal,gap='12px',align_items='center'),position='absolute',inset='0',background='rgba(14,16,18,0.45)',display='flex',justify_content='center',padding_top='96px')
    return div(bg+overlay,position='relative',height='900px',overflow='hidden')

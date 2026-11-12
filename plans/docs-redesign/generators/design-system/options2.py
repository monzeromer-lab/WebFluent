from spec2 import *
import ds2
JBM="'JetBrains Mono', ui-monospace, monospace"
INS="'Instrument Sans', Inter, sans-serif"; BRI="'Bricolage Grotesque', Inter, sans-serif"; NEWS="Newsreader, Georgia, serif"; INTER="Inter, system-ui, sans-serif"
GF_A='<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Instrument+Sans:wght@400..700&amp;display=swap">'
GF_C='<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,400..800&amp;family=Newsreader:ital,opsz,wght@0,6..72,400..700;1,6..72,400..700&amp;display=swap">'
GRID=dict(background_color='var(--bg)',background_image='linear-gradient(var(--line) 1px, transparent 1px), linear-gradient(90deg, var(--line) 1px, transparent 1px)',background_size='24px 24px',background_position='-1px -1px')
def hdr(T,active):
    nav=tabs(T,['Learn','Recipes','Reference','Errors','Playground'],active)
    srch=f'<label style="{st(display="flex",align_items="center",gap="8px",flex="0 1 260px",padding="0 10px",min_height="40px",border="1px solid var(--line)",border_radius=T["r_ctl"],background="var(--sunken)",color="var(--muted)")}"><input type="search" placeholder="Search the docs" aria-label="Search the docs" style="{st(border="0",background="transparent",font="14px "+F(T,"ui"),color="var(--ink)",flex="1",min_width="0",outline="none")}">{kbd(T,"⌘K")}</label>'
    return row(row(logo(30)+tag('span','WebFluent',font=f"700 16px {F(T,'display')}",letter_spacing='-.01em'),gap='10px')+div(nav,flex='1',display='flex',justify_content='center',align_self='flex-end')+srch,gap='24px',padding='12px 20px 0',border_bottom='1px solid var(--line)',background='var(--surface)',min_height='64px')
# ======================= A: Blueprint
def annotated(T,big=False):
    W,Hh=(760,300) if big else (640,260)
    btn=div(button(T,'Save changes'),position='absolute',left='50%',top='50%',transform='translate(-50%, -50%)')
    dim=div(div('',position='absolute',left='0',top='0',width='1px',height='9px',background='var(--accent-ink)')+div('',position='absolute',right='0',top='0',width='1px',height='9px',background='var(--accent-ink)')+div('',position='absolute',left='0',right='0',top='4px',height='1px',background='var(--accent-ink)')+tag('span','136',position='absolute',left='50%',top='-20px',transform='translateX(-50%)',font=f"500 12px {JBM}",color='var(--accent-ink)',background='var(--bg)',padding='0 4px'),position='absolute',left='calc(50% - 68px)',width='136px',top=f'calc(50% - 46px)',height='9px')
    vdim=div(div('',position='absolute',left='0',top='0',width='9px',height='1px',background='var(--accent-ink)')+div('',position='absolute',left='0',bottom='0',width='9px',height='1px',background='var(--accent-ink)')+div('',position='absolute',top='0',bottom='0',left='4px',width='1px',background='var(--accent-ink)')+tag('span','44',position='absolute',left='16px',top='50%',transform='translateY(-50%)',font=f"500 12px {JBM}",color='var(--accent-ink)'),position='absolute',left='calc(50% + 84px)',top='calc(50% - 22px)',height='44px',width='9px')
    def lead(text,pos,top,w,side='left'):
        line=div('',height='1px',width=f'{w}px',background='var(--accent-ink)',flex='none')
        dot=div('',width='7px',height='7px',border_radius='4px',background='var(--accent-ink)',flex='none')
        lab_=tag('span',text,font=f"500 12.5px {JBM}",color='var(--ink)',white_space='nowrap',background='var(--bg)',padding='2px 6px',border='1px solid var(--line)',border_radius='3px')
        inner=(lab_+line+dot) if side=='left' else (dot+line+lab_)
        p={'right':pos} if side=='left' else {'left':pos}
        return div(inner,position='absolute',top=top,display='flex',align_items='center',**p)
    anns=lead('tone: .primary','calc(50% + 72px)','calc(50% - 12px)',48 if not big else 120,'left')+lead('radius 4','calc(50% + 70px)','calc(50% + 34px)',60,'right')+lead('label: String','calc(50% + 2px)','calc(50% + 46px)',60,'left')
    return div(btn+dim+vdim+anns,position='relative',height=f'{Hh}px',border='1px solid var(--line)',border_radius=T['r_panel'],overflow='hidden',**GRID)
def extraA(T): return col(lab(T,'Signature: annotations. Live components are measured and labelled the way a technical drawing labels a part.')+annotated(T,True),gap='12px')
def ctxA(T):
    side=col(search_field(T)+''.join(nav_item(T,t,t=='Button',count=str(i+1)) for i,t in enumerate(['Container','Row','Grid','Button','IconButton','Input','Select'])),gap='2px',flex='0 1 220px',min_width='190px')
    props=''.join(row(tag('span',a,font=f"500 13px {JBM}",width='90px',flex='none')+tag('span',b,font=f"500 13px {JBM}",color='var(--accent-ink)',width='150px',flex='none')+tag('span',c,font=f"14px {F(T,'ui')}",color='var(--muted)'),gap='12px',padding='9px 0',border_bottom='1px solid var(--line)') for a,b,c in [('label','String','What it says'),('tone','.primary .danger …','Its colour'),('size','.sm .md .lg','Its height: 36, 44, 52')])
    main=col(row(chip(T,'Reference')+chip(T,'Since 3.0')+chip(T,'Web','accent')+chip(T,'App','accent'),gap='6px')+tag('h1','Button',margin='0',font=f"{T['h1w']} 40px/1.08 {F(T,'display')}",letter_spacing=T['h1track'])+tag('p','A control that does something when pressed.',margin='0',font=f"16px/1.6 {F(T,'prose')}",color='var(--muted)')+annotated(T)+div(props),gap='14px',flex='1 1 480px',min_width='0')
    rail=col(lab(T,'It can raise')+callout(T,'help')+callout(T,'note'),gap='14px',flex='0 1 220px',min_width='190px')
    return div(hdr(T,'Reference')+div(side+main+rail,display='flex',flex_wrap='wrap',gap='32px',padding='28px 24px',align_items='flex-start'),border='1px solid var(--line)',border_radius=T['r_panel'],overflow='hidden',background='var(--bg)')
def search_field(T): return f'<label style="{st(display="flex",align_items="center",padding="0 10px",min_height="38px",border="1px solid var(--line)",border_radius=T["r_ctl"],background="var(--sunken)",margin_bottom="10px")}"><input type="search" placeholder="Filter 77 built-ins" aria-label="Filter components" style="{st(border="0",background="transparent",font="13px "+F(T,"ui"),color="var(--ink)",flex="1",min_width="0",outline="none")}"></label>'
A=dict(id='A',option='Option A',name='Blueprint',fonts=dict(display=INS,ui=INS,prose=INS,mono=JBM),famnames=dict(display='Instrument Sans, 650',ui='Instrument Sans',prose='Instrument Sans',mono='JetBrains Mono'),
 h1w='650',h1track='-.03em',h2w='650',h2track='-.02em',btnw='600',r_ctl='4px',r_panel='4px',r_chip='3px',nav='top',callout='leader',prose_size='16px',
 roles=[('Ground','bg','The drawing board, with a 24px grid where things are measured'),('Sheet','surface','Panels and the current tab'),('Raised','sunken','Inputs and hovered rows'),('Ink','ink','Text and outlines'),('Action','brand','Primary buttons'),('Link','brand_ink','Links and references'),('Construction','accent_ink','Dimension lines, leaders and focus, nothing else'),('Error','danger','Diagnostics')],
 concept='The docs are drawn like a technical drawing of the language. Components sit on a faint measuring grid, cyan construction lines give their sizes, and labels on leader lines name each part, so a reference page shows exactly how a thing is built.',
 principles=['Cyan only ever measures or points: dimension lines, leaders, focus.','Instrument Sans for everything you read, JetBrains Mono for code and measurements.','Square corners and thin rules: the precision is the decoration.'],
 typenote='Instrument Sans, slightly condensed and sharp at 650, reads like a drafting hand; JetBrains Mono carries code and every number that measures.',
 scale=[('Display','56','1.02','650','display','-.035em','One language, measured'),('Heading 1','40','1.08','650','display','-.03em','State and reactivity'),('Heading 2','26','1.2','650','display','-.02em','What redraws'),('Heading 3','19','1.3','600','display','-.01em','Derived values'),('Body','16','1.65','400','prose','0','Only what reads a value draws again when it changes.'),('Small','13','1.45','500','ui','0','Since 3.0, in every output'),('Code','13','1.65','400','mono','0','Button("Save").primary')],
 shapenote='Corners of 4px everywhere; elevation is a hairline, never a shadow, except for overlays.',
 elev=[('Ground','none'),('Sheet','0 0 0 1px var(--line)'),('Overlay','0 4px 8px rgba(0,0,0,.5), 0 16px 40px rgba(0,0,0,.6)')],
 contextnote='The Button reference page: an annotated live preview, a filterable index of the built-ins, the active tab drawn as a sheet pulled forward.',extra=extraA)
# ======================= B: Periodic
CATS=[('Layout','brand',[('Co','Container'),('Rw','Row'),('Gr','Grid'),('St','Stack'),('Sp','Spacer')]),('Navigation','accent',[('Nb','Navbar'),('Sb','Sidebar'),('Lk','Link'),('Tb','Tabs'),('Mn','Menu')]),('Data','brand_subtle',[('Cd','Card'),('Ta','Table'),('Ls','List'),('Bd','Badge'),('Av','Avatar')]),('Forms','accent_subtle',[('In','Input'),('Se','Select'),('Sw','Switch'),('Sl','Slider'),('Fm','Form')]),('Feedback','danger_subtle',[('Al','Alert'),('To','Toast'),('Md','Modal'),('Sn','Spinner'),('Pg','Progress')]),('Documents','sunken',[('Dc','Document'),('Ch','Chart'),('Qr','QrCode'),('Sd','Slide'),('Pb','PageBreak')])]
FG={'brand':'#FFFFFF','accent':'#FFFFFF','brand_subtle':'var(--brand-ink)','accent_subtle':'var(--accent-ink)','danger_subtle':'var(--danger)','sunken':'var(--ink)'}
def ptile(ab,name,tone,size=72,on=False):
    v=tone.replace('_','-')
    return div(tag('span',ab,font=f"800 {int(size*0.36)}px/1 {JBM}",letter_spacing='-1px')+tag('span',name,font=f"600 {max(9,int(size*0.14))}px/1.1 {INTER}",color='#FFFFFF' if FG[tone]=='#FFFFFF' else 'var(--ink)',white_space='nowrap',overflow='hidden',text_overflow='ellipsis',max_width='100%'),width=f'{size}px',height=f'{size}px',border_radius=f'{round(size*12/64)}px',background=f'var(--{v})',color=FG[tone],display='flex',flex_direction='column',align_items='flex-start',justify_content='space-between',padding=f'{int(size*0.14)}px',box_sizing='border-box',outline='2px solid var(--accent-ink)' if on else 'none',outline_offset='2px')
def table(size=72):
    cols=''.join(col(tag('p',c,margin='0',font=f"600 13px {INTER}",color='var(--muted)')+''.join(ptile(a,n,t,size,on=(n=='Button')) for a,n in items),gap='8px') for c,t,items in CATS)
    return div(cols,display='flex',gap='12px',flex_wrap='wrap')
def extraB(T): return col(lab(T,'Signature: the periodic table. Every built-in has a tile in the logo’s shape, coloured by family; the home page and the reference index are this table.')+table(76),gap='12px')
def ctxB(T):
    hero=col(tag('h1','One language. Every output.',margin='0',font=f"800 52px/1.0 {JBM}",letter_spacing='-.045em',max_width='620px')+tag('p','A static site, an app, a PDF or a slide deck from the same typed file, checked before anyone sees it.',margin='0',font=f"18px/1.6 {INTER}",color='var(--muted)',max_width='520px')+row(button(T,'Start the tour')+button(T,'Open the playground','secondary'),gap='10px')+row(tag('code','curl -fsSL https://webfluent.dev/install.sh | sh',font=f"13px {JBM}",color='var(--ink)'),padding='10px 14px',border='1px solid var(--line)',border_radius=T['r_ctl'],background='var(--surface)',width='fit-content'),gap='20px',flex='1 1 460px')
    return div(hdr(T,'Learn')+div(hero+col(lab(T,'77 built-ins, one tile each')+table(58),gap='10px',flex='1 1 520px'),display='flex',flex_wrap='wrap',gap='40px',padding='40px 28px',align_items='flex-start'),border='1px solid var(--line)',border_radius=T['r_panel'],overflow='hidden',background='var(--bg)')
B=dict(id='B',option='Option B',name='Periodic',fonts=dict(display=JBM,ui=INTER,prose=INTER,mono=JBM),famnames=dict(display='JetBrains Mono, 800',ui='Inter',prose='Inter',mono='JetBrains Mono'),
 h1w='800',h1track='-.045em',h2w='800',h2track='-.035em',btnw='600',r_ctl='8px',r_panel='12px',r_chip='6px',nav='tile',callout='tile',prose_size='16px',
 roles=[('Canvas','bg','The dark ground'),('Surface','surface','Panels, one step up'),('Raised','sunken','Hovered rows; the Documents family'),('Ink','ink','Text and icons'),('Layout','brand','Tiles for layout; primary buttons'),('Navigation','accent','Tiles for navigation'),('Link','brand_ink','Links; the Data family'),('Error','danger','Diagnostics; the Feedback family')],
 concept='The logo’s tile becomes the whole system. Every built-in is a tile in the logo’s shape and proportion, coloured by its family like a periodic table, and headings are set in the logo’s own typeface. You learn the language’s parts by sight.',
 principles=['One tile per built-in, page kind and output, always in the logo’s 12-on-64 corner.','Headings in JetBrains Mono 800, tight like the mark; paragraphs in Inter.','Six families, six fills, all from the palette: no new colours.'],
 typenote='JetBrains Mono at 800 makes every heading look like it was cut from the logo; Inter keeps paragraphs easy to read.',
 scale=[('Display','56','1.0','800','display','-.045em','one language'),('Heading 1','40','1.05','800','display','-.04em','State and reactivity'),('Heading 2','26','1.15','800','display','-.035em','What redraws'),('Heading 3','19','1.3','700','ui','-.01em','Derived values'),('Body','16','1.7','400','prose','0','Only what reads a value draws again when it changes.'),('Small','13','1.45','500','ui','0','Since 3.0, in every output'),('Code','13','1.65','400','mono','0','Button("Save").primary')],
 shapenote='Every corner follows the logo’s proportion, so a tile, a panel and a control look cut from the same piece.',
 elev=[('Canvas','none'),('Surface','0 0 0 1px var(--line)'),('Overlay','0 4px 8px rgba(0,0,0,.5), 0 16px 40px rgba(0,0,0,.6)')],
 contextnote='The home page: the headline in the logo’s face beside the periodic table of built-ins; each tile opens its reference page.',extra=extraB)
# ======================= C: Nocturne
def extraC(T):
    para=tag('p','Nothing re-runs the whole page. The compiler ties each piece of text to the values it reads'+tag('sup','1',font=f"700 11px {F(T,'ui')}",color='var(--brand-ink)')+', so a change touches only what it must.',margin='0',font=f"400 19px/1.75 {F(T,'prose')}",flex='1 1 520px',max_width='600px')
    note=row(tag('span','1',font=f"700 12px {F(T,'ui')}",color='var(--brand-ink)',padding_top='3px')+tag('p','This is why a list of a thousand rows stays fast: one row changing redraws one row.',margin='0',font=f"italic 15px/1.55 {F(T,'prose')}",color='var(--muted)'),gap='8px',align_items='flex-start',flex='0 1 260px')
    return col(lab(T,'Signature: the margin. Notes sit beside the sentence they explain, numbered in reading order, so the chapter never stops for a box.')+wrap(para+note,gap='40px',align_items='flex-start'),gap='12px')
def ctxC(T):
    p=lambda t,sup='': tag('p',t+(tag('sup',sup,font=f"700 11px {F(T,'ui')}",color='var(--brand-ink)') if sup else ''),margin='0',font=f"400 19px/1.75 {F(T,'prose')}",color='var(--ink)',max_width='620px')
    side=col(''.join(nav_item(T,t,t=='State and reactivity',count=str(i+1)) for i,t in enumerate(['Getting started','A first page','Elements and flags','State and reactivity','Components','Stores'])),gap='2px',flex='0 1 200px',min_width='180px')
    main=col(row(tag('span','4',font=f"800 96px/0.9 {F(T,'display')}",color='var(--brand-ink)',letter_spacing='-.04em')+col(chip(T,'Tutorial','brand')+tag('h1','State and reactivity',margin='0',font=f"{T['h1w']} 40px/1.05 {F(T,'display')}",letter_spacing=T['h1track']),gap='8px',align_items='flex-start'),gap='20px',align_items='flex-end')
        +p('A value you declare with <code style="font-family:JetBrains Mono,monospace;font-size:.85em">state</code> is read by whatever shows it, and only what reads it draws again when it changes.','1')
        +codeblock(T)+p('Nothing re-runs the whole page. Click the button: the text that reads <code style="font-family:JetBrains Mono,monospace;font-size:.85em">count</code> changes, and nothing else does.','2'),gap='18px',flex='1 1 480px',min_width='0')
    rail=col(div('',height='150px')+callout(T,'note')+div('',height='250px')+callout(T,'help'),gap='0',flex='0 1 230px',min_width='190px')
    return div(hdr(T,'Learn')+div(side+main+rail,display='flex',flex_wrap='wrap',gap='36px',padding='36px 24px',align_items='flex-start'),border='1px solid var(--line)',border_radius=T['r_panel'],overflow='hidden',background='var(--bg)')
C=dict(id='C',option='Option C',name='Nocturne',fonts=dict(display=BRI,ui=BRI,prose=NEWS,mono=JBM),famnames=dict(display='Bricolage Grotesque, 750',ui='Bricolage Grotesque',prose='Newsreader',mono='JetBrains Mono'),
 h1w='750',h1track='-.025em',h2w='700',h2track='-.015em',btnw='650',r_ctl='10px',r_panel='14px',r_chip='999px',nav='rule',callout='margin',prose_size='19px',
 roles=[('Night','bg','The reading room'),('Surface','surface','Code plates and the header'),('Raised','sunken','Inputs and hovered rows'),('Ink','ink','Text, softened from white for long reading'),('Action','brand','Buttons'),('Link','brand_ink','Links, note numbers, chapter numerals'),('Code accent','accent_ink','Code, data and output'),('Error','danger','Diagnostics')],
 concept='The docs are a book you read at night beside the editor. A serif made for screens carries the chapters, an expressive grotesk names things and labels controls, and notes sit in the margin beside the line they explain instead of interrupting it.',
 principles=['Newsreader at 19px for anything longer than a sentence; Bricolage Grotesque for headings and controls.','Chapter numerals are big because the book is a sequence; nothing else is numbered.','Softer corners and roomier spacing than a tool: this is where people read.'],
 typenote='Newsreader is a screen serif with optical sizes, comfortable for long chapters on a dark page; Bricolage Grotesque gives headings a voice of their own.',
 scale=[('Display','56','1.02','800','display','-.03em','One language, read slowly'),('Heading 1','40','1.05','750','display','-.025em','State and reactivity'),('Heading 2','26','1.2','700','display','-.015em','What redraws'),('Heading 3','20','1.3','650','display','0','Derived values'),('Body','19','1.75','400','prose','0','Only what reads a value draws again when it changes.'),('Small','13','1.45','500','ui','0','Since 3.0, in every output'),('Code','13','1.65','400','mono','0','Button("Save").primary')],
 shapenote='Softer corners for panels and controls, fully round labels so they never read as buttons, generous space between sections.',
 elev=[('Night','none'),('Plate','0 0 0 1px var(--line)'),('Overlay','0 4px 8px rgba(0,0,0,.5), 0 16px 40px rgba(0,0,0,.6)')],
 contextnote='A chapter of the book: the numeral leads, the serif carries the reading, and the margin holds the asides next to their sentences.',extra=extraC)
H={'A':4720,'B':4720,'C':4560}
for T,ctxf,fn,link in [(A,ctxA,'DS2-A-Blueprint.dc.html',GF_A),(B,ctxB,'DS2-B-Periodic.dc.html',''),(C,ctxC,'DS2-C-Nocturne.dc.html',GF_C)]:
    body=sheet(T,ctxf(T))
    page(fn,f"Design system {T['option']}: {T['name']}",body,1440,H[T['id']],fonts_link=link)
    t=body.replace('{{modeLabel}}','Switch to Day')
    css=ds2.vars_css('.ds',ds2.NIGHT)+ds2.vars_css('.ds.day',ds2.DAY)
    fonts='@font-face{font-family:"Inter";font-weight:400 800;src:url("file:///home/monzer-omer/Code/WebFluent/docs/fonts/inter-latin.woff2")}@font-face{font-family:"JetBrains Mono";font-weight:400 800;src:url("file:///home/monzer-omer/Code/WebFluent/docs/fonts/jetbrains-mono-latin.woff2")}'
    open(f'twin2-{fn}.html','w').write(f'<!doctype html><html><head><meta charset="utf-8">{link.replace("&amp;","&")}<style>{fonts}body{{margin:0}}{css}.ds *{{box-sizing:border-box}}</style></head><body><div class="ds" style="width:1440px;background:var(--bg);color:var(--ink)">{t}</div><script>setTimeout(()=>document.title="H"+document.documentElement.scrollHeight,1500)</script></body></html>')
print('ok')

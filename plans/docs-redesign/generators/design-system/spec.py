from ds import *
MONO="'JetBrains Mono', ui-monospace, monospace"
def F(T,k): return T['fonts'][k]
def h2(T,t): return tag('h2',t,margin='0',font=f"{T['h2w']} 26px/1.2 {F(T,'display')}",letter_spacing=T['h2track'],color='var(--ink)')
def lab(T,t): return tag('p',t,margin='0',font=f"600 13px/1.4 {F(T,'ui')}",color='var(--muted)')
def section(T,title,intro,body):
    return tag('section',col(col(h2(T,title)+(tag('p',intro,margin='0',font=f"400 15px/1.55 {F(T,'ui')}",color='var(--muted)',max_width='640px') if intro else ''),gap='6px')+body,gap='20px'),padding='44px 0',border_top='1px solid var(--line)')
# --------- controls ---------
def button(T,label,kind='primary',size='md',disabled=False,icon=''):
    pad={'sm':'6px 12px','md':'10px 16px','lg':'13px 22px'}[size]; h={'sm':'36px','md':'44px','lg':'52px'}[size]; fs={'sm':'13px','md':'14px','lg':'16px'}[size]
    base=dict(font=f"{T['btnw']} {fs} {F(T,'ui')}",padding=pad,min_height=h,border_radius=T['r_ctl'],cursor='pointer',display='inline-flex',align_items='center',gap='8px',letter_spacing=T.get('btntrack','0'))
    if kind=='primary': base.update(background='var(--brand)',color='#FFFFFF',border='1px solid var(--brand)')
    elif kind=='secondary': base.update(background='var(--surface)',color='var(--ink)',border='1px solid var(--strong)' if T['id']=='C' else '1px solid var(--line)')
    elif kind=='ghost': base.update(background='transparent',color='var(--brand-ink)',border='1px solid transparent')
    elif kind=='danger': base.update(background='var(--danger)',color='#FFFFFF',border='1px solid var(--danger)')
    if T['id']=='B' and kind=='primary': base.update(box_shadow='inset 0 -2px 0 rgba(0,0,0,.25)')
    if disabled: base.update(opacity='.45',cursor='not-allowed')
    return tag('button',icon+label,attrs=' type="button"'+(' disabled=""' if disabled else ''),**base)
def field(T,label,value,hint=None,error=None,mono=False):
    bd='var(--danger)' if error else 'var(--line)'
    inp=f'<input type="text" value="{E(value)}" aria-label="{E(label)}" style="{st(font=("14px "+MONO) if mono else ("15px "+F(T,"ui")),padding="10px 12px",min_height="44px",border="1px solid "+bd,border_radius=T["r_ctl"],background="var(--surface)",color="var(--ink)",width="100%")}">'
    sub=''
    if hint: sub=tag('p',hint,margin='0',font=f"13px/1.4 {F(T,'ui')}",color='var(--muted)')
    if error: sub=tag('p',error,margin='0',font=f"600 13px/1.4 {F(T,'ui')}",color='var(--danger)',attrs=' role="alert"')
    return f'<label style="{st(display="flex",flex_direction="column",gap="6px",font="600 13px "+F(T,"ui"),color="var(--ink)",flex="1 1 240px")}">{label}{inp}{sub}</label>'
def switch(T,label,on=True):
    track=div(div('',width='18px',height='18px',border_radius='9px',background='#FFFFFF',margin_left='18px' if on else '2px',box_shadow='0 1px 2px rgba(0,0,0,.3)'),width='40px',height='22px',border_radius='11px',background='var(--brand)' if on else 'var(--strong)',display='flex',align_items='center',padding='0 0 0 0')
    return tag('button',track+tag('span',label),attrs=f' type="button" role="switch" aria-checked="{"true" if on else "false"}"',display='inline-flex',align_items='center',gap='10px',background='transparent',border='0',color='var(--ink)',font=f"14px {F(T,'ui')}",cursor='pointer',min_height='44px',padding='0')
def chip(T,t,tone='plain'):
    tones={'plain':('var(--sunken)','var(--muted)'),'brand':('var(--brand-subtle)','var(--brand-ink)'),'accent':('var(--accent-subtle)','var(--accent-ink)'),'danger':('var(--danger-subtle)','var(--danger)')}
    bg,fg=tones[tone]
    r=T['r_chip']
    return tag('span',t,display='inline-flex',align_items='center',gap='6px',padding='3px 9px',border_radius=r,font=f"600 12px {F(T,'ui')}",background=bg,color=fg,white_space='nowrap')
def kbd(T,t): return tag('kbd',t,font=f"500 12px {MONO}",padding='2px 6px',border='1px solid var(--line)',border_bottom='2px solid var(--line)',border_radius='4px',background='var(--surface)',color='var(--ink)')
def tabs(T,items,active):
    out=''
    for t in items:
        on=t==active
        if T['nav']=='caret':
            mark=tag('span','^'*max(2,len(t)),font=f"700 11px/1 {MONO}",color='var(--accent-ink)' if on else 'transparent',letter_spacing='-.5px',display='block',height='10px',overflow='hidden',white_space='nowrap',width='100%')
            out+=tag('button',tag('span',t)+mark,attrs=f' type="button" role="tab" aria-selected="{"true" if on else "false"}"',display='inline-flex',flex_direction='column',align_items='flex-start',gap='2px',background='transparent',border='0',padding='8px 4px 2px',font=f"{'600' if on else '500'} 14px {F(T,'ui')}",color='var(--ink)' if on else 'var(--muted)',cursor='pointer',min_height='44px')
        elif T['nav']=='tile':
            out+=tag('button',t,attrs=f' type="button" role="tab" aria-selected="{"true" if on else "false"}"',background='var(--brand-subtle)' if on else 'transparent',color='var(--brand-ink)' if on else 'var(--muted)',border='1px solid '+('var(--brand)' if on else 'transparent'),border_radius=T['r_ctl'],padding='8px 14px',font=f"600 13px {F(T,'ui')}",cursor='pointer',min_height='40px')
        else:
            out+=tag('button',t,attrs=f' type="button" role="tab" aria-selected="{"true" if on else "false"}"',background='transparent',border='0',border_bottom='2px solid '+('var(--brand-ink)' if on else 'transparent'),padding='10px 2px',font=f"{'600' if on else '500'} 15px {F(T,'ui')}",color='var(--ink)' if on else 'var(--muted)',cursor='pointer',min_height='44px')
    gap={'caret':'18px','tile':'4px','rule':'24px'}[T['nav']]
    return div(out,attrs=' role="tablist"',display='flex',gap=gap,border_bottom='1px solid var(--line)' if T['nav']!='tile' else '0',flex_wrap='wrap')
def codeblock(T):
    head=row(row(tag('span','Home.wf',font=f"500 12.5px {MONO}",color=SYN['ink'])+tag('span','Preview',font=f"500 12.5px {F(T,'ui')}",color=SYN['com'])+tag('span','HTML',font=f"500 12.5px {F(T,'ui')}",color=SYN['com']),gap='16px')+tag('button','Copy',attrs=' type="button"',font=f"600 12px {F(T,'ui')}",color=SYN['ink'],background='transparent',border='1px solid #3A4046',border_radius=T['r_ctl'],padding='5px 10px',min_height='32px',cursor='pointer'),justify_content='space-between',padding='8px 10px 8px 16px',border_bottom='1px solid #2C3035')
    body=tag('pre',hl(SAMPLE),margin='0',padding='16px 18px',font=f"13px/1.65 {MONO}",color=SYN['ink'],white_space='pre',overflow_x='auto')
    return div(head+body,background='var(--term)',border_radius=T['r_panel'],overflow='hidden',border='1px solid #2C3035')
def diagnostic(T):
    msg='error[T05]: `User` has no field `nmae`\n --> src/pages/Profile.wf:3:18\n  |\n3 |     Heading(user.nmae).h1\n  |                  ^^^^\n  = help: its fields are `id`, `name`'
    return div(tag('pre',E(msg),margin='0',padding='14px 16px',font=f"12.5px/1.6 {MONO}",color='#FFD7D5',white_space='pre',overflow_x='auto'),background='#2A1215',border='1px solid #5A2420',border_radius=T['r_panel'])
def callout(T,kind='note'):
    words={'note':('note','Coming from React? <code>useState</code> is <code>state</code>, and there is no setter: assign to it.','brand'),'help':('help','A derived value cannot be assigned. Change what it reads instead.','accent')}
    k,t,tone=words[kind]
    if T['callout']=='compiler':
        return row(tag('span',k+':',font=f"700 13px {MONO}",color=f'var(--{tone}-ink)',flex='none',padding_top='1px')+tag('p',t,margin='0',font=f"15px/1.55 {F(T,'ui')}",color='var(--ink)'),gap='12px',align_items='flex-start',padding='14px 16px',background=f'var(--{tone}-subtle)',border_radius=T['r_panel'])
    if T['callout']=='tile':
        return row(div('i' if kind=='note' else '?',width='28px',height='28px',flex='none',border_radius='6px',background=f'var(--{"brand" if tone=="brand" else "accent"})',color='#FFFFFF',display='flex',align_items='center',justify_content='center',font=f"800 14px {MONO}")+tag('p',t,margin='0',font=f"15px/1.55 {F(T,'ui')}"),gap='12px',align_items='flex-start',padding='14px 16px',background='var(--sunken)',border='1px solid var(--line)',border_radius=T['r_panel'])
    n='1' if kind=='note' else '2'
    return row(tag('span',n,font=f"700 12px/1 {F(T,'ui')}",color='var(--brand-ink)',padding_top='4px',flex='none')+tag('p',t,margin='0',font=f"italic 15.5px/1.6 {F(T,'prose')}",color='var(--ink)'),gap='10px',align_items='flex-start')
def nav_item(T,t,on=False,count=None):
    if T['nav']=='caret':
        return tag('a',tag('span','›' if on else '',color='var(--accent-ink)',font=f"700 14px {MONO}",width='10px',display='inline-block')+t,attrs=' href="#"',display='flex',gap='6px',align_items='center',padding='6px 8px',color='var(--ink)' if on else 'var(--muted)',font=f"{'600' if on else '400'} 14px {F(T,'ui')}",text_decoration='none',background='var(--sunken)' if on else 'transparent',border_radius=T['r_ctl'])
    if T['nav']=='tile':
        return tag('a',tag('span',t[:2],width='24px',height='24px',border_radius='6px',background='var(--brand)' if on else 'var(--sunken)',color='#FFFFFF' if on else 'var(--muted)',display='inline-flex',align_items='center',justify_content='center',font=f"800 10.5px {MONO}",flex='none')+t,attrs=' href="#"',display='flex',gap='10px',align_items='center',padding='5px 6px',color='var(--ink)' if on else 'var(--muted)',font=f"{'600' if on else '400'} 14px {F(T,'ui')}",text_decoration='none',border_radius=T['r_ctl'],background='var(--brand-subtle)' if on else 'transparent')
    return tag('a',(tag('span',count,font=f"600 12px {F(T,'ui')}",color='var(--brand-ink)' if on else 'var(--subtle)',width='22px',display='inline-block') if count else '')+t,attrs=' href="#"',display='flex',align_items='center',padding='6px 0',color='var(--ink)' if on else 'var(--muted)',font=f"{'600' if on else '400'} 15px {F(T,'ui')}",text_decoration='none')
def tile(T,ab,name,cat,tone='brand'):
    return col(div(ab,width='64px',height='64px',border_radius='12px',background=f'var(--{tone})',color='#FFFFFF',display='flex',align_items='center',justify_content='center',font=f"800 22px {MONO}",letter_spacing='-1px')+tag('p',name,margin='0',font=f"600 14px {F(T,'ui')}")+tag('p',cat,margin='0',font=f"13px {F(T,'ui')}",color='var(--muted)'),gap='6px',align_items='flex-start')
# --------- spec sheet ---------
def colors(T):
    roles=T['roles']
    sw=''
    for name,var,role in roles:
        d=DAY[var]; n=NIGHT[var]
        sw+=col(div('',height='72px',background=f'var(--{var.replace("_","-")})',border_radius=T['r_panel'],border='1px solid var(--line)')+tag('p',name,margin='0',font=f"600 14px {F(T,'ui')}")+tag('p',role,margin='0',font=f"13px/1.4 {F(T,'ui')}",color='var(--muted)')+tag('p',f'Day {d} / Night {n}',margin='0',font=f"12px {MONO}",color='var(--subtle)'),gap='6px',min_width='0')
    pairs=[('Ink on page','ink','bg'),('Muted on page','muted','bg'),('Link on page','brand_ink','bg'),('White on action','#FFFFFF','brand'),('Code on console','#E6E9EC','term')]
    ct=''
    for lbl,f,b in pairs:
        fd=f if f.startswith('#') else DAY[f]; bd=DAY[b]; fn=f if f.startswith('#') else NIGHT[f]; bn=NIGHT[b]
        ct+=row(tag('span',lbl,flex='1',font=f"14px {F(T,'ui')}")+tag('span',f'{cr(fd,bd):.1f}:1',font=f"600 13px {MONO}",color='var(--ok)')+tag('span',f'{cr(fn,bn):.1f}:1',font=f"600 13px {MONO}",color='var(--ok)',width='64px',text_align='right'),gap='16px',padding='8px 0',border_bottom='1px solid var(--line)')
    ct=col(row(tag('span','Text contrast, WCAG',flex='1',font=f"600 13px {F(T,'ui')}",color='var(--muted)')+tag('span','Day',font=f"600 13px {F(T,'ui')}",color='var(--muted)')+tag('span','Night',font=f"600 13px {F(T,'ui')}",color='var(--muted)',width='64px',text_align='right'),gap='16px')+ct,gap='0',flex='1 1 360px')
    return section(T,'Colour','The brand palette, unchanged. What changes between options is the job each colour does.',div(sw,display='grid',grid_template_columns='repeat(4, minmax(0, 1fr))',gap='20px 16px')+div(ct,max_width='560px'))
def typescale(T):
    rows=''
    for name,sz,lh,wt,fam,trk,sample in T['scale']:
        rows+=row(col(tag('span',name,font=f"600 13px {F(T,'ui')}",color='var(--ink)')+tag('span',f'{sz}px on {lh}, weight {wt}',font=f"12px {MONO}",color='var(--subtle)'),gap='2px',width='170px',flex='none')+tag('p',sample,margin='0',font=f"{wt} {sz}px/{lh} {F(T,fam)}",letter_spacing=trk,color='var(--ink)',flex='1',min_width='0'),gap='24px',align_items='baseline',padding='14px 0',border_bottom='1px solid var(--line)')
    fams=wrap(''.join(col(tag('p',role,margin='0',font=f"600 13px {F(T,'ui')}",color='var(--muted)')+tag('p',T['famnames'][k],margin='0',font=f"600 20px {F(T,k)}"),gap='4px',flex='1 1 200px') for role,k in [('Display','display'),('Interface','ui'),('Reading','prose'),('Code','mono')]),gap='24px',align_items='flex-start')
    return section(T,'Type',T['typenote'],fams+div(rows))
def shape(T):
    radii=wrap(''.join(col(div('',width='72px',height='72px',background='var(--brand-subtle)',border='1px solid var(--brand)',border_radius=r)+tag('span',f'{n}, {r}',font=f"12px {MONO}",color='var(--muted)'),gap='8px',align_items='flex-start') for n,r in [('Control',T['r_ctl']),('Panel',T['r_panel']),('Tile','12px'),('Chip',T['r_chip'])]),gap='24px',align_items='flex-end')
    sp=wrap(''.join(col(div('',width=f'{v}px',height=f'{v}px',background='var(--accent)',border_radius='2px')+tag('span',str(v),font=f"12px {MONO}",color='var(--muted)'),gap='6px',align_items='flex-start') for v in [4,8,12,16,24,32,48,64]),gap='18px',align_items='flex-end')
    el=wrap(''.join(col(div('',width='120px',height='72px',background='var(--surface)',border='1px solid var(--line)',border_radius=T['r_panel'],box_shadow=s)+tag('span',n,font=f"12px {F(T,'ui')}",color='var(--muted)'),gap='8px',align_items='flex-start') for n,s in T['elev']),gap='24px')
    return section(T,'Shape, space and depth',T['shapenote'],wrap(col(lab(T,'Radius')+radii,gap='12px',flex='1 1 340px')+col(lab(T,'Spacing, on a 4px step')+sp,gap='12px',flex='1 1 420px'),gap='40px',align_items='flex-start')+col(lab(T,'Depth')+el,gap='12px'))
def components(T):
    btns=col(lab(T,'Buttons')+wrap(button(T,'Install WebFluent')+button(T,'Read the book','secondary')+button(T,'Copy','ghost')+button(T,'Delete project','danger')+button(T,'Building…','primary',disabled=True),gap='10px')+wrap(button(T,'Small','primary','sm')+button(T,'Medium','primary','md')+button(T,'Large','primary','lg'),gap='10px'),gap='12px')
    forms=col(lab(T,'Fields')+wrap(field(T,'Project name','field-notes',hint='Lowercase, no spaces.')+field(T,'Email','ada@example',error='Enter a full address, like ada@example.com.'),gap='16px',align_items='flex-start')+wrap(switch(T,'Show compiled output',True)+switch(T,'Right to left',False)+row(tag('span','Search',font=f"14px {F(T,'ui')}",color='var(--muted)')+kbd(T,'⌘')+kbd(T,'K'),gap='6px'),gap='24px'),gap='12px')
    marks=col(lab(T,'Labels')+wrap(chip(T,'Tutorial','brand')+chip(T,'How-to','accent')+chip(T,'Reference')+chip(T,'Error','danger')+chip(T,'Since 3.0')+chip(T,'Web','accent')+chip(T,'PDF'),gap='8px'),gap='12px')
    nav=col(lab(T,'Tabs')+tabs(T,['Learn','Recipes','Reference','Errors','Playground'],'Reference'),gap='12px')
    side=col(lab(T,'Sidebar')+col(''.join(nav_item(T,t,t=='State and reactivity',count=str(i+1)) for i,t in enumerate(['Getting started','A first page','Elements and flags','State and reactivity','Components'])),gap='2px'),gap='12px',flex='0 1 280px')
    notes=col(lab(T,'Asides')+callout(T,'note')+callout(T,'help'),gap='10px',flex='1 1 360px')
    code=col(lab(T,'Code')+codeblock(T),gap='12px',flex='1 1 440px',min_width='0')
    diag=col(lab(T,'Diagnostic')+diagnostic(T),gap='12px',flex='1 1 360px',min_width='0')
    extra=T['extra'](T) if T.get('extra') else ''
    return section(T,'Components','The pieces every page is built from, drawn in this option’s hand.',btns+forms+wrap(marks+nav,gap='40px',align_items='flex-start')+wrap(side+notes,gap='40px',align_items='flex-start')+wrap(code+diag,gap='24px',align_items='flex-start')+extra)
def sheet(T,in_context):
    head=row(row(logo(56)+col(tag('p',T['option'],margin='0',font=f"600 14px {F(T,'ui')}",color='var(--brand-ink)')+tag('h1',T['name'],margin='0',font=f"{T['h1w']} 52px/1.02 {F(T,'display')}",letter_spacing=T['h1track']),gap='4px'),gap='20px')
        +tag('button',tag('span','{{modeLabel}}'),attrs=' type="button" onClick="{{toggleDark}}" aria-label="Switch between Day and Night"',font=f"600 14px {F(T,'ui')}",padding='10px 16px',min_height='44px',border_radius=T['r_ctl'],border='1px solid var(--line)',background='var(--surface)',color='var(--ink)',cursor='pointer'),justify_content='space-between',gap='24px',flex_wrap='wrap')
    intro=wrap(tag('p',T['concept'],margin='0',font=f"400 19px/1.6 {F(T,'prose')}",color='var(--ink)',flex='1 1 520px',max_width='680px')
        +col(''.join(row(div('',width='6px',height='6px',border_radius='3px',background='var(--accent-ink)',flex='none',margin_top='8px')+tag('p',p,margin='0',font=f"15px/1.55 {F(T,'ui')}",color='var(--muted)'),gap='12px',align_items='flex-start') for p in T['principles']),gap='12px',flex='1 1 380px'),gap='48px',align_items='flex-start')
    return div(col(head+intro,gap='28px',padding='56px 0 44px')+colors(T)+typescale(T)+shape(T)+components(T)+section(T,'In context',T['contextnote'],in_context),max_width='1280px',margin='0 auto',padding='0 64px 64px',font_family=F(T,'ui'))

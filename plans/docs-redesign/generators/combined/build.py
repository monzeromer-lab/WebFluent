import re
from refine import *
import pages
home=pages.home
from ui import page
P=[('Unified-Home.dc.html','WebFluent docs — Home',home,1440,{'tour':['wf','html','js']}),
('Unified-Learn.dc.html','Learn — State and reactivity',learn,1440,{'learn':['wf','html','js']}),
('Unified-Recipe.dc.html','Recipe — Fetch and show a list',recipe,1440,{'rec':['wf','preview','html','js']}),
('Unified-Reference.dc.html','Reference — Button',reference,1440,{'ref':['wf','html','css']}),
('Unified-Error.dc.html','Error — T05',error_page,1440,{}),
('Unified-ComingFrom.dc.html','Coming from React',coming,1440,{}),
('Unified-Search.dc.html','Search palette',search_page,1440,{}),
('Unified-Playground.dc.html','Playground',playground,1440,{}),
('Unified-Home-phone.dc.html','Home — phone',home,390,{'tour':['wf','html','js']}),
('Unified-Learn-phone.dc.html','Learn — phone',learn,390,{'learn':['wf','html','js']})]
import json,os
H={'Unified-Home.dc.html': 2520, 'Unified-Learn.dc.html': 1760, 'Unified-Recipe.dc.html': 1650, 'Unified-Reference.dc.html': 1760, 'Unified-Error.dc.html': 1340, 'Unified-ComingFrom.dc.html': 1570, 'Unified-Search.dc.html': 900, 'Unified-Playground.dc.html': 900, 'Unified-Home-phone.dc.html': 5560, 'Unified-Learn-phone.dc.html': 3780}
os.makedirs('twin',exist_ok=True)
for fn,title,f,w,blocks in P:
    body=f()
    page(fn,title,body,w,H[fn],blocks)
    # plain twin for measuring: unwrap sc-if (notes shown, first tab shown), fill holes
    t=body
    t=re.sub(r'<sc-if value="\{\{tabs\.\w+\.not_\w+\}\}" hint-placeholder-val="\{\{true\}\}">.*?</sc-if>','',t,flags=re.S)
    t=re.sub(r'<sc-if value="\{\{(quizRight|quizWrong)\}\}"[^>]*>.*?</sc-if>','',t,flags=re.S)
    t=re.sub(r'<sc-if value="[^"]*" hint-placeholder-val="\{\{false\}\}">.*?</sc-if>','',t,flags=re.S)
    t=re.sub(r'</?sc-if[^>]*>','',t)
    t=t.replace('{{count}}','3').replace('{{snippet}}','Button("Save").primary').replace('{{prevStyle}}','padding:10px 18px')
    t=re.sub(r'\{\{[^}]*\}\}','',t)
    from ui import CSSV,FONTS
    css=FONTS.replace('/_blob/d19d616a9bd5d5492d89a2df48f34586','file:///home/monzer-omer/Code/WebFluent/docs/fonts/inter-latin.woff2').replace('/_blob/384893c6de26fed90e3face153cb051c','file:///home/monzer-omer/Code/WebFluent/docs/fonts/jetbrains-mono-latin.woff2')
    open('twin/'+fn+'.html','w').write(f'<!doctype html><html><head><meta charset="utf-8"><style>{css} body{{margin:0}} {CSSV}</style></head><body><div class="wf-root" style="background: var(--bg); color: var(--ink); font-family: Inter, sans-serif">{t}</div><script>document.title="H"+document.documentElement.scrollHeight</script></body></html>')
    print(fn, w)

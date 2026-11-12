import re
from refine import *
from ref2 import reference_v2
import pages
home=pages.home
from ui import page
P=[('Unified-Home.dc.html','WebFluent docs — Home',home,1440,{'tour':['wf','html','js']}),
('Unified-Learn.dc.html','Learn — State and reactivity',learn,1440,{'learn':['wf','html','js']}),
('Unified-Recipe.dc.html','Recipe — Fetch and show a list',recipe,1440,{'rec':['wf','preview','html','js']}),
('Unified-Reference.dc.html','Reference — Button',reference_v2,1440,{}),
('Unified-Error.dc.html','Error — T05',error_page,1440,{}),
('Unified-ComingFrom.dc.html','Coming from React',coming,1440,{}),
('Unified-Search.dc.html','Search palette',search_page,1440,{}),
('Unified-Playground.dc.html','Playground',playground,1440,{}),
('Unified-Home-phone.dc.html','Home — phone',home,390,{'tour':['wf','html','js']}),
('Unified-Learn-phone.dc.html','Learn — phone',learn,390,{'learn':['wf','html','js']})]
import json,os
H={'Unified-Home.dc.html': 2510, 'Unified-Learn.dc.html': 1710, 'Unified-Recipe.dc.html': 1650, 'Unified-Reference.dc.html': 2480, 'Unified-Error.dc.html': 1360, 'Unified-ComingFrom.dc.html': 1520, 'Unified-Search.dc.html': 900, 'Unified-Playground.dc.html': 900, 'Unified-Home-phone.dc.html': 5480, 'Unified-Learn-phone.dc.html': 3460}
os.makedirs('twin',exist_ok=True)
def tx(b):
    for x,y in [('border-radius: 6px','border-radius: 4px'),('border-radius: 8px','border-radius: 4px'),('border-radius: 10px','border-radius: 4px'),('border-radius: 12px','border-radius: 4px'),('border-radius: 14px','border-radius: 6px'),('border-radius: 7px','border-radius: 4px'),('border-radius: 999px','border-radius: 3px'),('Inter, sans-serif',"'Instrument Sans', Inter, sans-serif"),('Inter, system-ui, sans-serif',"'Instrument Sans', Inter, sans-serif"),('Unified-','BP-')]:
        b=b.replace(x,y)
    import re
    b=re.sub(r' →(?=</a>)','',b)
    b=re.sub(r'(?<=>)(\d+) · ',r'\1. ',b)
    b=re.sub(r'\b([A-Z]\d{2,3}) · ',r'\1: ',b)
    b=b.replace(' · ',', ')
    return b
for fn,title,f,w,blocks in P:
    body=tx(f())
    fn=fn.replace('Unified-','BP-')
    page(fn,title,body,w,H[fn.replace('BP-','Unified-')],blocks)
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
    open('twin/'+fn+'.html','w').write(f'<!doctype html><html><head><meta charset="utf-8"><link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Instrument+Sans:wght@400..700&display=swap"><style>{css} body{{margin:0}} {CSSV}</style></head><body><div class="wf-root" style="background: var(--bg); color: var(--ink); font-family: Inter, sans-serif">{t}</div><script>setTimeout(()=>document.title="H"+document.documentElement.scrollHeight,1200)</script></body></html>')
    print(fn, w)

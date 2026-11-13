"""The Blueprint pages §7.5 listed as still to draw.

Showcase (T8), 404 (T9), an About essay (T2 variant), the Changelog, and the
Arabic right-to-left Learn chapter (T2) and Button reference (T4).

Run from any folder:  python3 extra.py OUT_DIR
It writes BP-<Name>.dc.html into OUT_DIR, built with the same helpers, the
same post-processing (build.tx) and the same page() wrapper as the other
artboards.
"""
import json, os, re, sys
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, '..', '..', '..', '..'))

import ui
from refine import *            # pages.*, ui.*, the refined live editor
import pages
from ref2 import stage, example, sbtn, toggle, seg_loop

VERSION = '6.0'
ui.NAV[5] = ('Showcase', 'Unified-Showcase.dc.html')
_logo = ui.logo
ui.logo = lambda: _logo().replace('>5.12<', f'>{VERSION}<')

# Extra view state for the Showcase filter; the rest of the script is the
# one every artboard carries.
ui.JS = ui.JS.replace("  device: s.device ?? 'desktop',", """  outAll: (s.out ?? 'all') === 'all', notOutAll: (s.out ?? 'all') !== 'all',
  outPdf: s.out === 'pdf', notOutPdf: s.out !== 'pdf',
  outSlides: s.out === 'slides', notOutSlides: s.out !== 'slides',
  showPdf: (s.out ?? 'all') !== 'slides', showSlides: (s.out ?? 'all') !== 'pdf',
  pickAll: () => this.setState({ out: 'all' }), pickPdf: () => this.setState({ out: 'pdf' }), pickSlides: () => this.setState({ out: 'slides' }),
  device: s.device ?? 'desktop',""")

def tx(b):
    for x, y in [('border-radius: 6px', 'border-radius: 4px'), ('border-radius: 8px', 'border-radius: 4px'), ('border-radius: 10px', 'border-radius: 4px'), ('border-radius: 12px', 'border-radius: 4px'), ('border-radius: 14px', 'border-radius: 6px'), ('border-radius: 7px', 'border-radius: 4px'), ('border-radius: 999px', 'border-radius: 3px'), ('Inter, sans-serif', "'Instrument Sans', Inter, sans-serif"), ('Inter, system-ui, sans-serif', "'Instrument Sans', Inter, sans-serif"), ('Unified-', 'BP-')]:
        b = b.replace(x, y)
    b = re.sub(r' →(?=</a>)', '', b)
    b = re.sub(r'(?<=>)(\d+) · ', r'\1. ', b)
    b = re.sub(r'\b([A-Z]\d{2,3}) · ', r'\1: ', b)
    b = b.replace(' · ', ', ')
    return b.replace('WebFluent 5.12.', f'WebFluent {VERSION}.')

# ------------------------------------------------------------------ pieces
def h1(t, **k):
    d = dict(font_size='clamp(32px, 4vw, 44px)', font_weight='800', letter_spacing='-.025em', line_height='1.1'); d.update(k)
    return txt(t, 'h1', **d)
def h2(t, i=None, **k):
    d = dict(font='650 24px/1.2 Inter, sans-serif', letter_spacing='-.02em', margin='18px 0 0'); d.update(k)
    return txt(t, 'h2', attrs=f' id="{i}"' if i else '', **d)
def prose(t, **k):
    d = dict(font_size='17px', line_height='1.75', max_width='680px'); d.update(k)
    return txt(t, **d)
def lede(t):
    return txt(t, font_size='17px', line_height='1.6', color='var(--muted)', max_width='680px')
def aside(t):
    """§7.3: a cyan dot and a leader line, never a boxed callout."""
    mark = row(div('', width='7px', height='7px', border_radius='4px', background='var(--accent-ink)', flex='none') + div('', width='28px', height='1px', background='var(--accent-ink)', flex='none'), gap='0', margin_top='10px', flex='none')
    return tag('aside', row(mark + txt(t, font_size='14.5px', line_height='1.65', color='var(--muted)', max_width='560px'), gap='12px', align_items='flex-start'), margin='4px 0')
def rail_links(title, items, cur=0):
    return label(title) + ''.join(a(t, '#', font_size='13.5px', color='var(--ink)' if i == cur else 'var(--muted)', padding='3px 0 3px 12px', border_left='1px solid ' + ('var(--accent-ink)' if i == cur else 'var(--line)')) for i, t in enumerate(items))
def side_item(t, on=False, href='#', extra=''):
    return a(row(tag('span', t, flex='1') + extra, gap='8px'), href, attrs_current=on, display='block', padding='6px 10px', font_size='14px', font_weight='600' if on else '400', color='var(--ink)' if on else 'var(--muted)', background='var(--surface)' if on else 'transparent', box_shadow='inset 2px 0 0 var(--accent-ink)' if on else 'none', border_radius='0 4px 4px 0' if on else '4px')
def code(t):
    return tag('code', E(t), font='500 .86em ' + MONO, background='var(--sunken)', border='1px solid var(--line)', border_radius='3px', padding='1px 5px')
def three(side, content, rail):
    return main(div(side + content + rail, display='flex', flex_wrap='wrap', gap='40px', align_items='flex-start'))
def ar_side(c, lbl):
    return tag('aside', c, attrs=f' aria-label="{lbl}"', flex='0 1 240px', min_width='210px', padding='28px 0')
def ar_rail(c, lbl='On this page'):
    return tag('aside', col(c, gap='8px'), attrs=f' aria-label="{lbl}"', flex='0 1 220px', min_width='200px', padding='32px 0')
def article(c):
    return tag('article', col(c, gap='20px'), flex='999 1 560px', min_width='0', padding='28px 0')

# ============================================================ T8 Showcase
BLOB = {
    'halyard-report': 'ad630b7d6fa314df5175bc1806207d17', 'invoice': '08801ee277f21eec40d1624368110375',
    'resume': '8ebc91c045a88f6c333fa0faf54a068e', 'arabic-workshop': 'f3aa388fb5592efc5458b58713faa063',
    'halyard-review': 'fc4da846ca3020e9608904494bd3a293', 'webfluent': '2626bfbcdc543fad3eb88665fed4237e',
    'webfluent-ar': '47c7c1ca2ba91fb666097e4eb9f34a3a', 'arabic-invoice': '06c0f6ebe3f16bee6a8865282d8f2afb',
}
RECIPE = {'document': 'Recipe: an invoice as a PDF', 'deck': 'Recipe: a deck from a list'}
RECIPE_BY = {'halyard-report': 'Recipe: a report with charts', 'resume': 'Recipe: a CV in Arabic', 'arabic-invoice': 'Recipe: an invoice as a PDF', 'invoice': 'Recipe: an invoice as a PDF', 'arabic-workshop': 'Recipe: a deck in Arabic', 'webfluent-ar': 'Recipe: a deck in Arabic', 'halyard-review': 'Recipe: a deck from a list', 'webfluent': 'Recipe: a deck from a list'}
def seg(items):
    out = ''
    for lbl, n, flag, pick in items:
        inner = tag('span', lbl) + tag('span', str(n), font='12px ' + MONO, color='var(--muted)')
        on = tag('button', inner, attrs=f' type="button" aria-pressed="true" onClick="{{{{{pick}}}}}"', font='600 13px Inter, sans-serif', display='inline-flex', gap='8px', align_items='center', padding='8px 14px', min_height='40px', border='0', border_radius='3px', background='var(--surface)', color='var(--ink)', box_shadow='0 0 0 1px var(--line)', cursor='pointer')
        off = tag('button', inner, attrs=f' type="button" aria-pressed="false" onClick="{{{{{pick}}}}}"', font='500 13px Inter, sans-serif', display='inline-flex', gap='8px', align_items='center', padding='8px 14px', min_height='40px', border='0', border_radius='3px', background='transparent', color='var(--muted)', cursor='pointer')
        nf = 'not' + flag[0].upper() + flag[1:]
        out += f'<sc-if value="{{{{{flag}}}}}" hint-placeholder-val="{{{{{"true" if flag=="outAll" else "false"}}}}}">{on}</sc-if><sc-if value="{{{{{nf}}}}}" hint-placeholder-val="{{{{{"false" if flag=="outAll" else "true"}}}}}">{off}</sc-if>'
    return div(out, attrs=' role="group" aria-label="Filter by output"', display='inline-flex', gap='2px', padding='3px', background='var(--sunken)', border='1px solid var(--line)', border_radius='4px')
def show_card(x):
    deck = x['kind'] == 'deck'
    ar = x['lang'] == 'ar'
    unit = 'slides' if deck else ('page' if x['pages'] == 1 else 'pages')
    alt = f"The first {'slide' if deck else 'page'} of {x['title']}"
    img = f'<img src="/_blob/{BLOB[x["slug"]]}" alt="{E(alt)}" loading="lazy" style="{st(width="100%", height="100%", object_fit="contain", object_position="center top", display="block", box_shadow="0 0 0 1px var(--line)")}">'
    prev = tag('div', div(img, height='100%', display='flex', justify_content='center'), attrs=' class="grid"', background_color='var(--bg)', height='240px', padding='20px 24px 0', border_bottom='1px solid var(--line)', overflow='hidden')
    lang_attr = ' dir="auto"'
    title = tag('h3', E(x['title']), attrs=lang_attr, margin='0', font='650 17px/1.35 Inter, sans-serif', letter_spacing='-.01em')
    desc = tag('p', E(x['description']), attrs=lang_attr, margin='0', font_size='14px', line_height='1.55', color='var(--muted)')
    facts = row(chip('Slides' if deck else 'PDF', 'brand') + tag('span', f"{x['pages']} {unit}", font='12.5px ' + MONO, color='var(--muted)') + tag('span', x['size'], font='12.5px ' + MONO, color='var(--muted)') + tag('span', 'Arabic, right to left' if ar else 'English', font_size='12.5px', color='var(--muted)'), gap='14px', flex_wrap='wrap')
    links = row(a(icon('doc', 14) + ('Open the deck' if deck else 'Open the PDF'), '#', display='inline-flex', gap='6px', align_items='center', font_weight='600', font_size='13.5px', color='var(--brand-ink)', min_height='32px')
                + a(icon('git', 14) + 'Source', '#', display='inline-flex', gap='6px', align_items='center', font_size='13.5px', color='var(--brand-ink)', min_height='32px')
                + a(RECIPE_BY[x['slug']], 'Unified-Recipe.dc.html', font_size='13.5px', color='var(--brand-ink)', min_height='32px', display='inline-flex', align_items='center'), gap='18px', flex_wrap='wrap', padding_top='12px', border_top='1px solid var(--line)')
    return tag('article', prev + col(title + desc + facts + links, gap='10px', padding='16px 18px 14px', flex='1'), display='flex', flex_direction='column', border='1px solid var(--line)', border_radius='4px', background='var(--surface)', overflow='hidden')
def showcase():
    data = json.load(open(os.path.join(REPO, 'site/src/showcase.json')))
    docs = [x for x in data if x['kind'] == 'document']; decks = [x for x in data if x['kind'] == 'deck']
    head = col(row(h1('Showcase') + copymenu(), justify_content='space-between', flex_wrap='wrap', gap='8px')
               + lede('Real files from real source. Each one is what <code>wf build</code> wrote from a folder under <code>examples/</code>, and every release builds it again, so what you see is what this version makes.'),
               gap='12px', max_width='760px')
    bar = row(row(tag('span', 'Output', font='600 13px Inter, sans-serif', color='var(--muted)') + seg([('All', len(data), 'outAll', 'pickAll'), ('PDF', len(docs), 'outPdf', 'pickPdf'), ('Slides', len(decks), 'outSlides', 'pickSlides')]), gap='10px')
              + txt('Websites and apps join when their examples have a page of their own to show.', font_size='13px', color='var(--muted)'),
              justify_content='space-between', gap='16px', flex_wrap='wrap', padding='14px 0', border_top='1px solid var(--line)', border_bottom='1px solid var(--line)')
    grid = lambda xs: div(''.join(show_card(x) for x in xs), display='grid', grid_template_columns='repeat(auto-fill, minmax(300px, 1fr))', gap='16px')
    sec = lambda t, s, xs, flag: f'<sc-if value="{{{{{flag}}}}}" hint-placeholder-val="{{{{true}}}}">' + tag('section', col(row(txt(t, 'h2', font='650 22px/1.2 Inter, sans-serif', letter_spacing='-.02em') + tag('span', s, font='13px ' + MONO, color='var(--muted)'), gap='12px', align_items='baseline') + grid(xs), gap='16px'), margin_top='36px') + '</sc-if>'
    more = row(txt('Add yours: an example is a folder under <code>examples/</code> with a <code>webfluent.app.json</code>. A pull request that adds one adds its card here, built by the same release as everything else.', font_size='14px', color='var(--muted)', max_width='720px') + abtn(icon('git', 15) + 'Browse the examples', '#', font_size='13.5px'), justify_content='space-between', gap='16px', flex_wrap='wrap', margin_top='40px', padding_top='20px', border_top='1px solid var(--line)')
    body = col(head + bar, gap='24px') + sec('Documents', f'{len(docs)} PDFs', docs, 'showPdf') + sec('Slide decks', f'{len(decks)} decks', decks, 'showSlides') + more
    return header('Showcase') + main(div(body, padding='32px 0 0')) + footer()

# ================================================================ T9 404
def not_found():
    addr = row(tag('span', 'You asked for', font_size='14px', color='var(--muted)') + tag('code', '/reference/<span style="text-decoration: underline wavy var(--danger); text-underline-offset: 4px">buton</span>', font='500 15px ' + MONO, background='var(--sunken)', border='1px solid var(--line)', border_radius='4px', padding='4px 8px'), gap='10px', flex_wrap='wrap')
    head = col(row(tag('span', '404', font='600 13px ' + MONO, color='var(--danger)', padding='2px 7px', border='1px solid var(--danger)', border_radius='3px') + tag('span', 'Not found', font='600 13px Inter, sans-serif', color='var(--muted)'), gap='10px')
               + h1('Nothing here') + addr
               + lede('No page has this address. Every address the old docs used redirects to its new home, so this one was most likely mistyped, or copied from a link that was cut short.'), gap='14px', max_width='720px')
    close = [('Button', '/reference/button', 'Reference', 'One letter away: a control that does something when pressed.'),
             ('ButtonGroup', '/reference/buttongroup', 'Reference', 'Buttons that belong together.'),
             ('Disable Save while it runs', '/recipes/disable-while-saving', 'How-to', 'A recipe that uses Button.')]
    def cl(t, p, k, s, first):
        return a(row(col(row(txt(t, font='650 16px Inter, sans-serif') + mode(k), gap='10px') + tag('code', p, font='13px ' + MONO, color='var(--muted)') + txt(s, font_size='14px', color='var(--muted)'), gap='6px', flex='1') + (tag('kbd', 'Enter', font='11px ' + MONO, border='1px solid var(--line)', border_radius='3px', padding='2px 6px', color='var(--muted)') if first else ''), gap='12px', align_items='flex-start'), 'Unified-Reference.dc.html' if k == 'Reference' else 'Unified-Recipe.dc.html', display='block', padding='14px 16px', background='var(--surface)' if first else 'transparent', box_shadow='inset 2px 0 0 var(--accent-ink)' if first else 'none', border_bottom='1px solid var(--line)')
    closest = col(row(txt('The closest pages', 'h2', font='650 20px/1.2 Inter, sans-serif') + txt('by how the address is spelled', font_size='13px', color='var(--muted)'), gap='12px', align_items='baseline', flex_wrap='wrap')
                  + div(''.join(cl(*c, i == 0) for i, c in enumerate(close)), border='1px solid var(--line)', border_radius='4px', overflow='hidden'), gap='12px')
    search = col(txt('Search instead', 'h2', font='650 20px/1.2 Inter, sans-serif')
                 + f'<label style="{st(display="flex", align_items="center", gap="10px", padding="0 14px", min_height="48px", border="1px solid var(--line)", border_radius="4px", background="var(--sunken)", color="var(--muted)")}">{icon("search", 16)}<input type="search" value="buton" aria-label="Search the docs" style="{st(border="0", background="transparent", font="15px Inter, sans-serif", color="var(--ink)", flex="1", min_width="0", outline="none")}"><kbd style="{st(font="11px " + MONO, border="1px solid var(--line)", border_radius="3px", padding="1px 6px")}">⌘K</kbd></label>'
                 + txt('Search reads the words you would use too: <code>onClick</code> finds <code>on click</code>, <code>useState</code> finds <code>state</code>, and a code like <code>T05</code> goes straight to its page.', font_size='14px', color='var(--muted)', line_height='1.6'), gap='12px')
    secs = [('Learn', 'The book, from an empty folder to a deployed site.', 'Unified-Learn.dc.html'), ('Recipes', 'One task each, with the complete code first.', 'Unified-Recipe.dc.html'), ('Reference', 'Every built-in, command and setting.', 'Unified-Reference.dc.html'), ('Errors', 'Every code the compiler can print, and its fix.', 'Unified-Error.dc.html'), ('Playground', 'The compiler, running in this page.', 'Unified-Playground.dc.html'), ('Showcase', 'PDFs and decks built from the examples.', 'Unified-Showcase.dc.html')]
    sections = col(txt('Or start from a section', 'h2', font='650 20px/1.2 Inter, sans-serif') + div(''.join(a(col(txt(t, font='650 15px Inter, sans-serif') + txt(s, font_size='13.5px', color='var(--muted)', line_height='1.5'), gap='4px'), h, display='block', padding='14px 16px', border='1px solid var(--line)', border_radius='4px', background='var(--surface)') for t, s, h in secs), display='grid', grid_template_columns='repeat(auto-fill, minmax(300px, 1fr))', gap='10px'), gap='12px')
    report = txt('Followed a link from this site? <a href="#">Report the broken link</a> and say where it was.', font_size='14px', color='var(--muted)')
    body = col(head + div(closest + search, display='grid', grid_template_columns='repeat(auto-fit, minmax(380px, 1fr))', gap='32px', align_items='start') + sections + report, gap='40px', padding='48px 0 0', max_width='1080px')
    return header() + main(body) + footer()

# ======================================================= T2 variant: About
ESSAYS = ['Why WebFluent', 'How reactivity works', 'Accessibility the compiler gives you', 'The security model', 'What a build ships', 'Static or single-page', 'Upgrading, 5 to 6', 'Contributing']
def graph():
    node = lambda kind, name, sub: col(tag('span', kind, font='12px ' + MONO, color='var(--muted)') + tag('span', name, font='600 14px ' + MONO, color='var(--ink)') + tag('span', sub, font_size='12.5px', color='var(--muted)'), gap='3px', padding='12px 14px', border='1px solid var(--line)', border_radius='4px', background='var(--surface)', min_width='170px')
    edge = lambda t: col(tag('span', t, font='11.5px ' + MONO, color='var(--muted)') + row(div('', height='1px', flex='1', background='var(--muted)') + '<svg aria-hidden="true" width="8" height="10" viewBox="0 0 8 10"><path d="M0 0l8 5-8 5z" fill="var(--muted)"></path></svg>', gap='0'), gap='4px', flex='1', min_width='60px', align_items='center')
    g = row(node('state', 'count', 'what you change') + edge('read by') + node('derived', 'double', 'worked out from it') + edge('read by') + node('text node', '"{double} items"', 'the one thing redrawn'), gap='8px', align_items='center')
    cap = txt('What the compiler draws for the example above: three nodes and two edges. Setting <code>count</code> walks the edges and touches nothing else on the page.', font_size='13.5px', color='var(--muted)', line_height='1.55')
    return tag('figure', col(tag('div', g, attrs=' class="grid"', background_color='var(--bg)', padding='28px 24px', border='1px solid var(--line)', border_radius='4px', overflow_x='auto') + tag('figcaption', cap), gap='10px'), margin='4px 0')
def about():
    side = tag('nav', col(col(label('About') + txt('Essays on why the language works the way it does. Read them once, in any order.', font_size='13px', color='var(--muted)', line_height='1.5', padding='0 10px'), gap='6px') + col(''.join(side_item(t, t == 'How reactivity works') for t in ESSAYS), gap='2px') + div('', height='1px', background='var(--line)', margin='4px 0') + a(icon('book', 14) + 'The book', 'Unified-Learn.dc.html', display='inline-flex', gap='8px', align_items='center', font_size='13.5px', color='var(--brand-ink)', padding='4px 10px'), gap='16px'), attrs=' aria-label="Essays"', flex='0 1 240px', min_width='210px', padding='28px 0')
    meta = row(tag('span', '9 minutes', font_size='13px', color='var(--muted)') + tag('span', 'Updated for 6.0', font_size='13px', color='var(--muted)') + a('Monzer Omer', '#', font_size='13px', color='var(--muted)'), gap='16px', flex_wrap='wrap')
    src = '''page Cart(path: "/cart", title: "Cart") {
    state count = 3
    derived double = count * 2

    Text("{double} items")
    Button("Add").primary { on click { count = count + 1 } }
}'''
    content = article(
        row(wrap(mode('About') + chip('Web') + chip('App'), gap='8px') + copymenu(), justify_content='space-between', flex_wrap='wrap', gap='8px')
        + h1('How reactivity works')
        + lede('WebFluent has no render function and no virtual DOM. The compiler works out, from the source, which piece of the page reads which value, and ties them together. This essay argues that is the right default, says what it costs, and says when you will notice.')
        + meta
        + h2('The question every framework answers', 'question')
        + prose('A value changes. What on the screen has to change with it? There are two ways to find out. One is to draw everything again and compare the result with what is there; that is what React does, and it is why a React component is a function that runs again. The other is to know in advance who reads what, and to tell only those readers.')
        + prose('WebFluent takes the second way, and takes it at build time. A page is not a function that runs again; it runs once, and what it leaves behind is a set of small subscriptions.')
        + h2('What the compiler knows', 'knows')
        + prose('Here is a page with one value you change and one value worked out from it:')
        + pre(src, border_radius='4px', border='1px solid #2C3035')
        + prose('The compiler reads <code>Text("{double} items")</code>, sees that it reads <code>double</code>, and sees that <code>double</code> reads <code>count</code>. Those are the only edges in the graph:')
        + graph()
        + aside('Solid and Svelte 5 make the same bet with signals. The difference here is that the graph is known before the page ships, so the compiler can check it: an effect that writes what it reads, every run, is <a href="Unified-Error.dc.html">X02</a>, refused at build time rather than found by a frozen tab.')
        + h2('What it costs', 'costs')
        + prose('A value is followed only where WebFluent can see it being read. A plain script under <code>src/</code> that reads <code>count</code> once gets a number, not a subscription. When a script needs to follow a value, hand it the value through <code>update:</code>, which runs again whenever what it reads changes, or hand it an action to call.')
        + prose('The other cost is that a <code>state</code> is read once, when it is made. That is what you want almost always, and the one time it is not, the compiler says so.')
        + h2('When you will notice', 'notice')
        + prose('Two places, both with a finding of their own. A <code>state</code> made from <code>now</code> holds the moment the page opened and never ticks; the fix is <code>derived</code>, and <a href="Unified-Error.dc.html">X06</a> offers it. A <code>for</code> with no <code>by</code> whose items hold state starts those items again when the list changes; give it a key, and <a href="Unified-Error.dc.html">U09</a> says which.')
        + prose('Everywhere else, the right thing redraws and nothing else does, and you do not have to think about it. That is the point.')
        + col(label('Where to go next') + ''.join(a(t, h, font_size='15px', color='var(--brand-ink)') for t, h in [('Learn 4: State and reactivity', 'Unified-Learn.dc.html'), ('Reference: derived', 'Unified-Reference.dc.html'), ('X06: a state that never ticks', 'Unified-Error.dc.html')]), gap='8px', margin_top='12px', padding_top='18px', border_top='1px solid var(--line)')
        + row(abtn(icon('back', 15) + 'Why WebFluent', '#') + abtn('The security model' + icon('arrow', 15), '#', 'primary'), justify_content='space-between', flex_wrap='wrap', gap='10px', margin_top='12px'))
    rail = tag('aside', col(rail_links('On this page', ['The question every framework answers', 'What the compiler knows', 'What it costs', 'When you will notice'], 1), gap='8px'), attrs=' aria-label="On this page"', flex='0 1 220px', min_width='200px', padding='32px 0')
    return header('Learn') + three(side, content, rail) + footer()

# ============================================================== Changelog
def md_inline(s):
    s = E(s)
    s = re.sub(r'\*\*(.+?)\*\*', r'<b>\1</b>', s)
    s = re.sub(r'`([^`]+)`', r'<code>\1</code>', s)
    s = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'<a href="#">\1</a>', s)
    return s
def release_notes():
    text = open(os.path.join(REPO, 'RELEASE_NOTES.md')).read()
    out = []
    for blk in re.split(r'^# WebFluent v', text, flags=re.M)[1:]:
        ver, _, rest = blk.partition(' Release Notes\n')
        parts = re.split(r'^## ', rest, flags=re.M)
        intro = ' '.join(p.replace('\n', ' ').strip() for p in parts[0].strip().split('\n\n')[:1])
        secs = []
        for p in parts[1:]:
            name, _, body = p.partition('\n')
            items = [re.sub(r'\s+', ' ', re.split(r'\n\s+- ', it)[0]).strip() for it in re.split(r'^- ', body, flags=re.M)[1:]]
            secs.append((name.strip(), [it.split('\n')[0] for it in items]))
        out.append((ver.strip(), intro, secs))
    return out
DATES = {'6.0.0': '13 November 2026', '5.14.0': '12 November 2026', '5.13.0': '11 November 2026', '5.12.0': '3 November 2026', '5.11.0': '29 October 2026', '5.10.0': '24 October 2026', '5.9.0': '22 October 2026', '5.8.0': '19 October 2026'}
def first_sentence(s, n=230):
    s = re.sub(r'\s*\(#\d+\)', '', s)
    m = re.match(r'(.*?\.)(\s|$)', s.replace('e.g.', 'e·g·'))
    s = (m.group(1) if m else s).replace('e·g·', 'e.g.')
    return s if len(s) <= n else s[:n].rsplit(' ', 1)[0] + '…'
def changelog():
    rel = release_notes()
    def version(v, intro, secs, full, latest=False):
        head = col(row(row(txt(v, 'h2', attrs=f' id="{v}"', font='650 30px/1.1 ' + MONO, letter_spacing='-.02em') + a(f'#{v}', '#', font='13px ' + MONO, color='var(--muted)') + (chip('Latest', 'brand') if latest else ''), gap='12px', align_items='baseline') + tag('time', DATES.get(v, ''), attrs=f' datetime="{v}"', font_size='13.5px', color='var(--muted)'), justify_content='space-between', flex_wrap='wrap', gap='8px')
                   + prose(md_inline(intro), font_size='16.5px', color='var(--ink)'), gap='10px')
        body = ''
        shown = secs if full else [s for s in secs if s[0] in ('Additions',)][:1]
        for name, items in shown:
            lim = 4 if full else 3
            lis = ''.join(tag('li', md_inline(first_sentence(it)), font_size='15px', line_height='1.65', margin_bottom='6px') for it in items[:lim])
            rest = len(items) - lim
            body += col(row(txt(name, 'h3', font='650 17px/1.3 Inter, sans-serif') + tag('span', str(len(items)), font='12px ' + MONO, color='var(--muted)'), gap='10px', align_items='baseline')
                        + tag('ul', lis, margin='0', padding_left='20px', max_width='720px')
                        + (a(f'{rest} more in {name.lower()}', '#', font_size='13.5px', color='var(--brand-ink)') if rest > 0 else ''), gap='8px')
        total = sum(len(i) for _, i in secs)
        tail = '' if full else row(btn(f'Show all {total} changes', font_size='13.5px', min_height='38px', padding='8px 14px') + a('Upgrading to ' + v, '#', font_size='13.5px', color='var(--brand-ink)'), gap='16px', flex_wrap='wrap')
        return tag('section', col(head + body + tail, gap='18px'), attrs=f' aria-labelledby="{v}"', padding='28px 0', border_top='1px solid var(--line)')
    vs = rel[:3]
    side_vs = ''
    for i, (v, _, _) in enumerate(rel[:14]):
        side_vs += side_item(tag('span', v, font='13.5px ' + MONO), i == 0, extra=tag('span', DATES.get(v, '').replace(' 2026', ''), font_size='12px', color='var(--muted)'))
    side = tag('nav', col(row(label('Versions') + txt(f'{len(rel)}', font='12px ' + MONO, color='var(--muted)'), justify_content='space-between', padding='0 10px') + col(side_vs, gap='2px') + a(f'All {len(rel)} versions', '#', font_size='13.5px', color='var(--brand-ink)', padding='4px 10px'), gap='12px'), attrs=' aria-label="Versions"', flex='0 1 232px', min_width='200px', padding='28px 0')
    head = col(row(txt('<a href="Unified-Reference.dc.html" style="color: var(--muted); text-decoration: none">Reference</a> / Changelog', font_size='13px', color='var(--muted)') + copymenu(), justify_content='space-between', flex_wrap='wrap', gap='8px')
               + h1('Changelog')
               + lede('What changed in each release, newest first. Every version has its own address, <code>/changelog#6.0.0</code>, and every feature on this site carries the version it arrived in.')
               + row(a(icon('git', 14) + 'Releases on GitHub', '#', display='inline-flex', gap='6px', align_items='center', font_size='13.5px', color='var(--brand-ink)') + a(icon('doc', 14) + 'RELEASE_NOTES.md', '#', display='inline-flex', gap='6px', align_items='center', font_size='13.5px', color='var(--brand-ink)') + a('Feed', '#', font_size='13.5px', color='var(--brand-ink)'), gap='18px', flex_wrap='wrap'), gap='12px')
    content = tag('article', col(head + version(*vs[0], True, True) + version(*vs[1], False) + version(*vs[2], False), gap='8px'), flex='999 1 560px', min_width='0', padding='28px 0')
    secs0 = [s for s, _ in vs[0][2]]
    rail = tag('aside', col(rail_links('In 6.0.0', secs0, 0) + div('', height='12px') + label('Upgrading') + txt('A project that builds with 5.14 builds with 6.0. What breaks on purpose is the Rust API.', font_size='13px', color='var(--muted)', line_height='1.55') + a('Upgrading, 5 to 6', '#', font_size='13.5px', color='var(--brand-ink)'), gap='8px'), attrs=' aria-label="On this page"', flex='0 1 220px', min_width='200px', padding='32px 0')
    return header('Reference') + three(side, content, rail) + footer()

# ============================================================ Arabic (RTL)
AR = json.load(open(os.path.join(REPO, 'site/src/translations/ar.json')))
SHELL = {
    'Learn': 'تعلّم', 'Recipes': 'وصفات', 'Reference': AR['nav.group.reference'], 'Errors': 'الأخطاء', 'Playground': 'ساحة التجربة', 'Showcase': AR['nav.showcase'],
    'Search the docs': AR['search.placeholder'], 'Building': 'تبني', 'Website': 'موقعًا', 'Web app': 'تطبيق ويب', 'PDF': 'PDF', 'Slides': 'شرائح', 'Custom elements': 'عناصر مخصّصة', 'Desktop / phone': 'تطبيقًا لسطح المكتب أو الهاتف',
    'What you are building': 'ما الذي تبنيه', 'Toggle dark mode': AR['theme.light'], 'Main': 'الرئيسي',
    'The tour': 'الجولة', 'The book': 'الكتاب', 'About essays': 'مقالات', 'Do': 'أنجِز', 'Coming from…': 'قادم من…', 'Look up': 'ابحث عن',
    'Components': AR['ref.index'], 'CLI and config': 'سطر الأوامر والإعدادات', 'For agents': 'للوكلاء', 'MCP endpoint': 'نقطة MCP',
    f'WebFluent {VERSION}. Every example on this site is compiled by the release it documents.': f'WebFluent {VERSION}. كل مثال في هذا الموقع يترجمه الإصدار الذي يوثّقه.',
    'Copy page': 'انسخ الصفحة', 'Open the Markdown twin': 'افتح نسخة Markdown', 'Copy': AR['code.copy'], 'Copy code': AR['code.copy'],
    'Open in Playground': 'افتحه في ساحة التجربة', 'Break it': 'اكسره', 'Apply the fix': 'طبّق الإصلاح',
    'Compiled in your browser, WebAssembly': 'تُرجم في متصفحك، WebAssembly', '1 error, the preview keeps the last good page': 'خطأ واحد، والمعاينة تُبقي آخر صفحة سليمة',
    'A name nothing declares': 'اسم لم يُعلنه شيء', 'Read T13': 'اقرأ T13', ', the same page `wf explain T13` prints': '، الصفحة نفسها التي يطبعها wf explain T13',
    'Tone': 'النغمة', 'Size': 'الحجم', 'Outlined': 'بحدّ', 'Pill': 'كبسولة', 'Measure': 'قِس',
}
def shell_ar(html):
    def tn(m):
        s = m.group(1); k = s.strip()
        return '>' + s.replace(k, SHELL[k]) + '<' if k in SHELL else m.group(0)
    html = re.sub(r'>([^<>]+)<', tn, html)
    html = re.sub(r'(aria-label|placeholder)="([^"]*)"', lambda m: f'{m.group(1)}="{SHELL.get(m.group(2), m.group(2))}"', html)
    return html
def header_ar(active):
    h = shell_ar(header(active)).replace('<span class="hide-sm">تبني</span>', '').replace('max-width: 360px', 'max-width: 280px')
    lang = a(tag('span', 'English', attrs=' lang="en"'), '#', attrs_current=False, display='inline-flex', align_items='center', font_weight='600', min_height='40px', padding='0 10px', border='1px solid var(--line)', border_radius='4px')
    return h.replace('<a href="#" style="color: var(--ink); text-decoration: none; font-size: 14px; display: inline-flex; gap: 6px', lang + '<a href="#" style="color: var(--ink); text-decoration: none; font-size: 14px; display: inline-flex; gap: 6px', 1)
def logical(html):
    """Physical sides to logical ones, so the same styles mirror under dir=rtl."""
    for x, y in [('border-left:', 'border-inline-start:'), ('border-right:', 'border-inline-end:'), ('margin-left:', 'margin-inline-start:'), ('margin-right:', 'margin-inline-end:'), ('padding-left:', 'padding-inline-start:'), ('padding-right:', 'padding-inline-end:'), ('text-align: left', 'text-align: start'), ('text-align: right', 'text-align: end'), ('inset 2px 0 0 var(--accent-ink)', 'inset -2px 0 0 var(--accent-ink)')]:
        html = html.replace(x, y)
    html = html.replace('border-radius: 0 4px 4px 0', 'border-radius: 4px 0 0 4px')
    html = html.replace('padding: 3px 0 3px 12px', 'padding: 3px 12px 3px 0')
    # arrows point the way the reader reads
    html = re.sub(r'(<svg aria-hidden="true" width="\d+" height="\d+" viewBox="0 0 24 24")( fill="none"[^>]*>)(<path d="M5 12h14M13 6l6 6-6 6"></path>|<path d="M19 12H5M11 6l-6 6 6 6"></path>)', r'\1 style="transform: scaleX(-1)"\2\3', html)
    # Arabic words inside a left-to-right island read right to left; .md stays .md
    html = html.replace('">تُرجم في متصفحك', '" dir="rtl">تُرجم في متصفحك').replace('">خطأ واحد،', '" dir="rtl">خطأ واحد،')
    html = html.replace('</svg>.md</button>', '</svg><bdi dir="ltr">.md</bdi></button>')
    # code stays left to right
    html = re.sub(r'<(pre|code|kbd)(?=[ >])', r'<\1 dir="ltr"', html)
    return html
def ltr(c):
    return f'<div dir="ltr">{c}</div>'

def learn_ar():
    chs = [('1', 'الشروع في العمل', True, 'done'), ('2', 'صفحتك الأولى', True, 'done'), ('3', 'العناصر والأعلام', True, 'done'), ('4', AR['ch.12'], False, 'now'), ('5', AR['ch.15'], False, ''), ('6', AR['ch.16'], False, ''), ('7', 'البيانات والموارد', False, ''), ('8', 'التصميم والسمات', False, ''), ('9', 'النماذج', False, ''), ('10', AR['ch.37'], False, ''), ('11', AR['ch.34'], False, '')]
    items = ''
    for n, t, tour, stt in chs:
        on = stt == 'now'
        items += a(row(tag('span', icon('check', 13) if stt == 'done' else n, width='22px', height='22px', display='inline-flex', align_items='center', justify_content='center', border_radius='11px', font='600 11px ' + MONO, background='var(--ok-subtle)' if stt == 'done' else ('var(--brand)' if on else 'var(--sunken)'), color='var(--ok)' if stt == 'done' else ('#fff' if on else 'var(--muted)'), flex='none') + tag('span', t, flex='1') + (chip(icon('play', 10) + 'جولة', 'accent', padding='1px 7px', font_size='11px') if tour else ''), gap='10px'), '#', attrs_current=on, display='block', padding='7px 8px', border_radius='6px', background='var(--surface)' if on else 'transparent', box_shadow='inset 2px 0 0 var(--accent-ink)' if on else 'none', color='var(--ink)', font_weight='600' if on else '400')
    abouts = ''.join(a(x, '#', color='var(--muted)', padding='5px 8px', display='block', font_size='13.5px') for x in ['كيف تعمل التفاعلية', 'نموذج الأمان', 'كيف يُخطَّط ملف PDF', 'ما الذي يشحنه البناء'])
    side = ar_side(col(col(row(label('الكتاب') + txt('4 من 11', font_size='12px', color='var(--muted)'), justify_content='space-between') + div(div('', width='31%', height='100%', background='var(--brand)', border_radius='3px'), height='6px', background='var(--line)', border_radius='3px'), gap='8px') + col(items, gap='2px') + col(label('مقالات') + abouts, gap='4px'), gap='16px'), 'فصول الكتاب')
    p = lambda t: prose(t, line_height='1.9')
    quiz = card(col(row(icon('check', 16) + txt('تحقّق من فهمك', font_weight='700', font_size='15px'), gap='8px')
        + txt('في صفحةٍ <code>state n = 0</code> و<code>derived d = n * 2</code>، وتعرض <code>Text("{d}")</code>. تُسند <code>n = 5</code>. ما الذي يُعاد رسمه؟', font_size='15px', line_height='1.8')
        + col(''.join(f'<label style="{st(display="flex", gap="10px", align_items="center", padding="12px 14px", border="1px solid var(--line)", border_radius="8px", font_size="14px", cursor="pointer", min_height="44px")}"><input type="radio" name="quiz" onChange="{{{{pick{i}}}}}"> {o}</label>' for i, o in enumerate(['تُرسم الصفحة كلها من جديد', 'النص الذي يقرأ <code>d</code> وحده', 'لا شيء، حتى تستدعي <code>render()</code>'])), gap='8px')
        + row(tag('button', 'تحقّق من الإجابة', attrs=' type="button" onClick="{{check}}"', font='600 14px Inter, sans-serif', padding='10px 16px', min_height='44px', border_radius='6px', background='var(--brand)', color='#fff', border='1px solid var(--brand)', cursor='pointer')
             + '<sc-if value="{{quizRight}}" hint-placeholder-val="{{false}}">' + chip(icon('check', 12) + 'صحيح: لا يُعاد رسم إلا ما يقرأ d، وهو يقرأ n من خلاله.', 'accent') + '</sc-if>'
             + '<sc-if value="{{quizWrong}}" hint-placeholder-val="{{false}}">' + chip(icon('x', 12) + 'ليس تمامًا. أعد قراءة «ما الذي يُعاد رسمه» أعلاه.', 'danger') + '</sc-if>', gap='12px', flex_wrap='wrap'), gap='14px'), padding='22px', background='var(--sunken)')
    content = article(
        row(wrap(chip('درس', 'brand') + chip('الفصل 4') + chip('الويب، التطبيق، PDF'), gap='8px') + row(a(tag('span', 'Read in English', attrs=' lang="en"'), '#', font_size='13px', color='var(--brand-ink)') + copymenu(), gap='12px'), justify_content='space-between', flex_wrap='wrap', gap='8px')
        + h1(AR['ch.12'], letter_spacing='0', line_height='1.3')
        + p('هذا الفصل هو الأصعب على أغلب القرّاء، لذا يمضي ببطء. القيمة التي تعلنها بـ<code>state</code> يقرؤها كل ما يعرضها، ولا يُعاد رسم إلا ما يقرؤها حين تتغيّر.')
        + col(txt('جرّبها', 'h2', font_size='20px', font_weight='800') + ltr(live_editor2('learn')), gap='12px')
        + txt('ما الذي يُعاد رسمه', 'h2', font_size='24px', font_weight='800', margin_top='12px')
        + p('لا شيء يعيد تشغيل الصفحة كلها. يرى المترجم أن <code>Text("Clicked {count} times")</code> يقرأ <code>count</code>، فيربط عقدة النص تلك وحدها به. انقر <b>+1</b> أعلاه: يتغيّر ذلك السطر وحده.')
        + aside('قادم من React؟ <code>useState</code> هنا <code>state</code>، ولا دالة ضبط: أسند إليها مباشرة. <a href="Unified-ComingFrom.dc.html">ورقة المقارنة</a>')
        + txt('القيم المشتقة', 'h2', font_size='24px', font_weight='800', margin_top='12px')
        + p('ما تحسبه من حالة يُكتب <code>derived</code>، ولا يُسند إليه أبدًا. يُعاد حسابه حين يتغيّر ما يقرؤه، فيبقى صحيحًا دون أن تتذكّره.')
        + pre('state items: [Item] = []\nderived total = items.map(i => i.price).sum()\n\nText("المجموع: {total:.currency}")', border_radius='4px', border='1px solid #2C3035')
        + quiz
        + row(abtn(icon('back', 15) + '3. العناصر والأعلام', '#') + abtn('5. ' + AR['ch.15'] + icon('arrow', 15), '#', 'primary'), justify_content='space-between', flex_wrap='wrap', gap='10px', margin_top='12px'))
    rail = ar_rail(rail_links(AR['docs.onthispage'], ['جرّبها', 'ما الذي يُعاد رسمه', 'القيم المشتقة', 'تحقّق من فهمك'], 1)
        + div('', height='12px') + label('أخطاء قد تواجهها هنا') + ''.join(a(row(chip(c, 'danger') + tag('span', t, font_size='13px', color='var(--muted)'), gap='8px'), 'Unified-Error.dc.html') for c, t in [('X01', 'إسناد إلى قيمة مشتقة'), ('X06', 'حالة لا تتقدّم أبدًا'), ('T17', 'await خارج إجراء')]), AR['docs.onthispage'])
    return header_ar('Learn') + three(side, content, rail) + shell_ar(footer())

def reference_ar():
    fams = [('الإجراءات', ['Button', 'IconButton', 'ButtonGroup', 'Dropdown']), ('التخطيط', ['Container', 'Row', 'Stack', 'Grid', 'Spacer', 'Divider']), ('التنقّل', ['Navbar', 'Sidebar', 'Link', 'Tabs', 'Menu']), ('النماذج', ['Input', 'Select', 'Checkbox', 'Switch', 'Slider', 'Form'])]
    idx = f'<label style="{st(display="flex", align_items="center", gap="8px", padding="0 10px", min_height="38px", border="1px solid var(--line)", border_radius="4px", background="var(--sunken)", color="var(--muted)", margin_bottom="14px")}">{icon("search", 14)}<input type="search" placeholder="{AR["ref.filter"].replace("{count}", "77")}" aria-label="تصفية المكوّنات" style="{st(border="0", background="transparent", font="13.5px Inter, sans-serif", color="var(--ink)", flex="1", min_width="0", outline="none")}"><kbd style="{st(font="11px " + MONO, color="var(--muted)")}">/</kbd></label>'
    for g, xs in fams:
        idx += row(txt(g, font='600 12.5px Inter, sans-serif', color='var(--muted)') + txt(str(len(xs)), font='12px ' + MONO, color='var(--muted)'), justify_content='space-between', margin='14px 0 4px', padding='0 10px')
        idx += ''.join(side_item(tag('span', x, attrs=' dir="ltr"'), x == 'Button') for x in xs)
    side = tag('nav', idx, attrs=' aria-label="المكوّنات المدمجة"', flex='0 1 232px', min_width='200px', padding='28px 0')
    title = col(row(txt(f'<a href="#" style="color: var(--muted); text-decoration: none">{AR["nav.group.reference"]}</a> / الإجراءات', font_size='13px', color='var(--muted)') + row(a(tag('span', 'Read in English', attrs=' lang="en"'), '#', font_size='13px', color='var(--brand-ink)') + copymenu(), gap='12px'), justify_content='space-between', flex_wrap='wrap', gap='8px')
        + txt('Button', 'h1', attrs=' dir="ltr"', font='650 44px/1.05 Inter, sans-serif', letter_spacing='-.03em', text_align='start')
        + txt('عنصر تحكّم يفعل شيئًا حين يُضغط. ما يعرضه يذهب في كتلته، وما يفعله يذهب في <code>on click</code>.', font_size='17px', line_height='1.8', color='var(--muted)', max_width='640px')
        + wrap(chip('منذ 3.0') + chip(icon('check', 11) + 'الويب', 'accent') + chip(icon('check', 11) + 'التطبيق', 'accent') + chip('ليس في PDF ولا الشرائح', 'off') + a(AR['showcase.source'], '#', font_size='13px', color='var(--brand-ink)', margin_left='4px'), gap='6px'), gap='12px')
    stg = stage().replace('role="group"', 'role="group" dir="ltr"')
    ex = div(example('أرسل نموذجًا', 'داخل Form، يرسله <code>type: .submit</code>؛ وأي زر آخر لا يرسله أبدًا.', col(div('', width='180px', height='36px', border='1px solid var(--line)', border_radius='4px', background='var(--surface)') + sbtn('انضم'), gap='10px', align_items='flex-start'), 'Form {\n    Input(bind: email, label: "البريد")\n    Button("انضم", type: .submit).primary\n}')
        + example('أثناء الحفظ', 'الإجراء غير المتزامن معلّق ما دام يعمل؛ اقرأه لتعطيل الزر.', sbtn('جارٍ الحفظ…', opacity='.55', cursor='not-allowed'), 'Button("احفظ", disabled: save.pending)\n    .primary { on click { save() } }')
        + example('خياران', 'يجمع ButtonGroup أزرارًا تنتمي معًا.', row(sbtn('نهار', 'quiet', border_radius='4px 0 0 4px') + sbtn('ليل', border_radius='0 4px 4px 0', margin_left='-1px'), gap='0'), 'ButtonGroup {\n    Button("نهار")\n    Button("ليل").primary\n}'), display='flex', flex_direction='column', gap='14px')
    c = lambda t: f'<code style="{st(font="500 12.5px " + MONO, background="var(--sunken)", border="1px solid var(--line)", border_radius="3px", padding="1px 5px")}">{E(t)}</code>'
    props = [('label', 'String', '', 'ما يقوله الزر. موضعية.'), ('tone', 'Tone', '.primary', 'واحدة من ستّ نغمات، أولها ' + c('.primary')), ('size', 'Size', '.md', 'ارتفاعه 36 أو 44 أو 52 بكسل، من ' + c('.sm') + ' إلى ' + c('.lg')), ('outlined', 'Bool', 'false', 'حدّ بلا تعبئة'), ('pill', 'Bool', 'false', 'طرفان مستديران بالكامل'), ('full', 'Bool', 'false', 'يملأ عرض صفّه'), ('type', 'ButtonType', '.button', c('.submit') + ' يرسل النموذج الذي هو فيه'), ('disabled', 'Bool', 'false', 'يُبهته ويتجاهل النقرات')]
    th = lambda h: f'<th scope="col" style="{st(text_align="start", padding="10px 14px", font="600 12.5px Inter, sans-serif", color="var(--muted)", border_bottom="1px solid var(--line)")}">{h}</th>'
    td = lambda x, **k: f'<td style="{st(padding="11px 14px", border_bottom="1px solid var(--line)", **k)}">{x}</td>'
    rows = ''.join('<tr>' + td(f'<span dir="ltr">{n}</span>', font='500 13px ' + MONO, color='var(--ink)', white_space='nowrap') + td(tag('span', E(t), attrs=' dir="ltr"', font='500 12px ' + MONO, color='var(--brand-ink)', background='var(--brand-subtle)', padding='2px 6px', border_radius='3px'), white_space='nowrap') + td(f'<span dir="ltr">{E(d)}</span>' if d else 'مطلوبة', font='13px ' + MONO, color='var(--muted)', white_space='nowrap') + td(w, font='14px Inter, sans-serif', color='var(--ink)', line_height='1.7') + '</tr>' for n, t, d, w in props)
    ptab = f'<div style="{st(overflow_x="auto", border="1px solid var(--line)", border_radius="4px", background="var(--surface)")}"><table style="{st(width="100%", border_collapse="collapse")}"><thead><tr>{th(AR["ref.prop"])}{th(AR["ref.type"])}{th("الافتراضي")}{th("ماذا تفعل")}</tr></thead><tbody>{rows}</tbody></table></div>'
    a11y = col(''.join(row(div('', width='6px', height='6px', background='var(--accent-ink)', flex='none', margin_top='11px', border_radius='1px') + txt(t, font_size='15px', line_height='1.8'), gap='12px', align_items='flex-start') for t in ['يُرسم <code>&lt;button type="button"&gt;</code> حقيقيًا، فتعمل Tab وEnter وSpace بلا شيفرة إضافية.', 'يحتاج الزر نصًا يراه القارئ أو يسمعه؛ والزر الخالي منهما هو <a href="Unified-Error.dc.html">A05</a>.', 'يجب أن يحتوي <code>aria-label</code> على الكلمات الظاهرة، وإلا لم يجده التحكّم الصوتي: <a href="Unified-Error.dc.html">A15</a>.']), gap='10px')
    hh = lambda t, i: txt(t, 'h2', attrs=f' id="{i}"', font='650 24px/1.3 Inter, sans-serif', margin='18px 0 0')
    content = tag('article', col(title + stg + hh('أمثلة', 'examples') + ex + hh(AR['ref.tab.props'], 'props') + ptab + hh(AR['ref.events'], 'events')
        + txt('كل حدث DOM، على عنصر <code>&lt;button&gt;</code> نفسه: <code>on click</code> و<code>on focus</code> و<code>on key("mod+s")</code>. المعالج المكتوب على استدعاء مكوّن يصل إلى جذره، فيُنقر زرّك المنسّق أيضًا.', font_size='15px', line_height='1.8', max_width='680px')
        + hh(AR['ch.22'], 'a11y') + a11y, gap='18px'), flex='999 1 560px', min_width='0', padding='28px 0 0')
    rail = ar_rail(rail_links(AR['docs.onthispage'], ['المعاينة', 'أمثلة', AR['ref.tab.props'], AR['ref.events'], AR['ch.22']], 0)
        + div('', height='12px') + label('التركيب') + pre('ButtonGroup\n├─ Button\n└─ Button\n\nForm\n└─ Button(type: .submit)', font='12.5px/1.6 ' + MONO, padding='12px 14px', border_radius='4px')
        + div('', height='12px') + label('ذو صلة') + ''.join(a(t, '#', font_size='13.5px', color='var(--brand-ink)') for t in ['<span dir="ltr">IconButton</span>', '<span dir="ltr">ButtonGroup</span>', '<span dir="ltr">Form</span>', 'وصفة: عطّل زر الحفظ أثناء عمله']), AR['docs.onthispage'])
    return header_ar('Reference') + three(side, content, rail) + shell_ar(footer())

def rtl_doc(doc):
    doc = doc.replace('<html lang="en">', '<html lang="ar" dir="rtl">', 1)
    return doc.replace('<div class="{{rootClass}}" style="background: var(--bg); color: var(--ink); font-family: Inter, system-ui, sans-serif; min-height: 100vh">',
                       '<div class="{{rootClass}}" dir="rtl" lang="ar" style="background: var(--bg); color: var(--ink); font-family: \'Instrument Sans\', \'Noto Sans Arabic\', \'Segoe UI\', Tahoma, system-ui, sans-serif; min-height: 100vh">', 1)

# ------------------------------------------------------------------ build
PAGES = [
    ('BP-Showcase.dc.html', 'Showcase', showcase, False, {}),
    ('BP-NotFound.dc.html', '404: not found', not_found, False, {}),
    ('BP-About.dc.html', 'About: how reactivity works', about, False, {}),
    ('BP-Changelog.dc.html', 'Changelog', changelog, False, {}),
    ('BP-Learn-ar.dc.html', 'تعلّم: الحالة والتفاعلية', learn_ar, True, {'learn': ['wf', 'html', 'js']}),
    ('BP-Reference-ar.dc.html', 'المرجع: Button', reference_ar, True, {}),
]
def build(out, heights):
    os.makedirs('root/project', exist_ok=True)
    for fn, title, f, rtl, blocks in PAGES:
        body = tx(f())
        if rtl:
            body = logical(shell_ar(body))
        doc = page(fn, title, body, 1440, heights.get(fn, 1600), blocks)
        if rtl:
            doc = rtl_doc(doc)
        open(os.path.join(out, fn), 'w').write(doc)
        print(fn)
if __name__ == '__main__':
    out = os.path.abspath(sys.argv[1])
    heights = json.loads(sys.argv[2]) if len(sys.argv) > 2 else {}
    build(out, heights)

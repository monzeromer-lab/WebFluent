#!/usr/bin/env python3
"""A bigger copy of a project: its guide pages repeated K times under new
names and routes, to see how a check, a build or the language server scales.

Usage: scripts/bench/grow.py SOURCE_PROJECT DEST K
"""
import os,re,shutil,sys
src,dst,k=sys.argv[1],sys.argv[2],int(sys.argv[3])
shutil.rmtree(dst,ignore_errors=True); shutil.copytree(src,dst,ignore=shutil.ignore_patterns("build"))
gd=os.path.join(dst,"src/pages/guide")
files=[f for f in os.listdir(gd) if f.endswith(".wf")]
for c in range(1,k):
    for f in files:
        s=open(os.path.join(gd,f)).read()
        s=re.sub(r'\bpage (\w+)\(path: "([^"]+)"',lambda m:f'page {m.group(1)}X{c}(path: "{m.group(2)}-x{c}"',s)
        s=re.sub(r'(translates:\s*)(\w+)',lambda m:f'{m.group(1)}{m.group(2)}X{c}',s)
        open(os.path.join(gd,f"x{c}_"+f),"w").write(s)
print(sum(1 for r,_,fs in os.walk(os.path.join(dst,"src")) for f in fs if f.endswith(".wf")),"files")

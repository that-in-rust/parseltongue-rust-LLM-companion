#!/usr/bin/env python3
"""Mutation runs: break real files, measure how many UNAFFECTED truth items the cutter still cuts exactly, and whether the damage is grey."""
import subprocess, sys, random, collections, os
TRUTH='/home/ubuntu/spanbench/truth/target/release/truth'; CUT='/home/ubuntu/spanbench/spancut/target/release/spancut'
random.seed(7)
def run(b,f): return subprocess.run([b,f],capture_output=True,text=True,timeout=60).stdout
def lc_to_off(src,lc):
    l,c=map(int,lc.split(':')); lines=src.split('\n'); return sum(len(x)+1 for x in lines[:l-1])+c
def truth_items(src,f):
    out=run(TRUTH,f); 
    if out.startswith('PARSE_ERROR'): return None
    return [(k,nm,lc_to_off(src,a),lc_to_off(src,b)) for k,nm,v,a,b in (l.split('\t') for l in out.splitlines())]
def cut_items(src_path):
    src=open(src_path).read(); out=run(CUT,src_path)
    return [(k,nm,lc_to_off(src,a),lc_to_off(src,b),float(c)) for k,nm,v,a,b,c in (l.split('\t') for l in out.splitlines())]
def mutate(src,items,kind):
    if kind=='delete_close_brace':
        pos=[i for i,ch in enumerate(src) if ch=='}']; 
        if not pos: return None
        p=random.choice(pos); return src[:p]+' '+src[p+1:], p, p+1
    if kind=='truncate_mid_item':
        it=random.choice(items); p=random.randint(it[2]+1,max(it[2]+1,it[3]-1)); return src[:p], p, len(src)
    if kind=='insert_half_item':
        it=random.choice(items); p=it[3]; ins='\npub fn half_typed(\n'; return src[:p]+ins+src[p:], p, p+len(ins)
    if kind=='delete_semicolon':
        pos=[i for i,ch in enumerate(src) if ch==';'];
        if not pos: return None
        p=random.choice(pos); return src[:p]+' '+src[p+1:], p, p+1
    if kind=='unbalanced_open_brace':
        it=random.choice(items); p=it[2]; return src[:p]+'{\n'+src[p:], p, p+2
files=[l.strip() for l in open(sys.argv[1]) if l.strip()]; random.shuffle(files); files=files[:int(sys.argv[2]) if len(sys.argv)>2 else 40]
KINDS=['delete_close_brace','truncate_mid_item','insert_half_item','delete_semicolon','unbalanced_open_brace']
stats={k:collections.Counter() for k in KINDS}; examples={k:[] for k in KINDS}
for f in files:
    src=open(f).read(); items=truth_items(src,f)
    if not items or len(items)<3: continue
    for kind in KINDS:
        r=mutate(src,items,kind)
        if not r: continue
        msrc,a,b=r; tmp='/tmp/mut.rs'; open(tmp,'w').write(msrc)
        cuts=cut_items(tmp); shift=len(msrc)-len(src)
        def adj(off): return off if off<a else off+shift
        def adj_end(off): return off if off<=a else off+shift
        unaffected=[(k,nm,s,e) for k,nm,s,e in items if e<=a or s>=b]
        affected=[(k,nm,s,e) for k,nm,s,e in items if not (e<=a or s>=b)]
        cutmap={(s):(k,nm,e,c) for k,nm,s,e,c in cuts}
        ok=0
        for k,nm,s,e in unaffected:
            c=cutmap.get(adj(s))
            if c and c[2]==adj_end(e) and c[3]>=0.9: ok+=1
        st=stats[kind]; st['files']+=1; st['unaffected']+=len(unaffected); st['unaffected_exact']+=ok
        # damage region: is there a grey/error span overlapping it?
        grey=[x for x in cuts if x[4]<0.5 and not (x[3]<=a or x[2]>=b+shift)]
        st['damage_flagged_grey']+= 1 if grey or kind=='truncate_mid_item' and any(x[4]<0.5 for x in cuts) else 0
        st['grey_spans_total']+=sum(1 for x in cuts if x[4]<0.5)
        if ok<len(unaffected) and len(examples[kind])<3:
            miss=[(k,nm) for k,nm,s,e in unaffected if not (cutmap.get(adj(s)) and cutmap[adj(s)][2]==adj_end(e) and cutmap[adj(s)][3]>=0.9)]
            examples[kind].append(f'{os.path.basename(f)} @{a}: lost {len(unaffected)-ok}/{len(unaffected)} e.g. {miss[:3]}')
for k in KINDS:
    st=stats[k]; u=st['unaffected'] or 1
    print(f"{k:24s} files {st['files']:3d}  unaffected items kept exact {st['unaffected_exact']}/{st['unaffected']} = {100*st['unaffected_exact']/u:.1f} %   damage flagged grey {st['damage_flagged_grey']}/{st['files']}   grey spans total {st['grey_spans_total']}")
    for e in examples[k]: print('     ',e)

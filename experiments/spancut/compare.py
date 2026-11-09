#!/usr/bin/env python3
"""Compare spancut vs syn truth on files. Usage: compare.py <filelist>  → summary + mismatch examples."""
import subprocess, sys, collections
TRUTH='/home/ubuntu/spanbench/truth/target/release/truth'; CUT=__import__('os').environ.get('CUT','/home/ubuntu/spanbench/spancut/target/release/spancut')
KMAP={'macro':'macro','extern_block':'externblock','extern_crate':'externcrate','trait_alias':'trait'}
def run(b,f):
    try: return subprocess.run([b,f],capture_output=True,text=True,timeout=60).stdout
    except Exception as e: return 'ERR '+str(e)
files=[l.strip() for l in open(sys.argv[1]) if l.strip()]
tot=collections.Counter(); ex=collections.defaultdict(list)
for f in files:
    tr=run(TRUTH,f); 
    if tr.startswith('PARSE_ERROR') or tr.startswith('ERR'): tot['files_unparseable']+=1; continue
    cu=run(CUT,f); tot['files']+=1
    T=[l.split('\t') for l in tr.splitlines()]; C=[l.split('\t') for l in cu.splitlines()]
    # syn item spans do NOT include outer attributes/docs? (syn Item span covers attrs: yes for items via Spanned) — compare end + start of keyword tolerant
    tmap={}
    for k,name,v,a,b in T:
        tmap[(a)]=(k,name,v,a,b)
    cmap={}
    for k,name,v,a,b,conf in C:
        if k in ('use',): pass
        cmap[a]=(k,name,v,a,b,float(conf))
    tot['truth_items']+=len(T); tot['cut_items']+=len(C)
    for a,(k,name,v,_,b) in tmap.items():
        c=cmap.get(a)
        if not c: tot['start_missed']+=1; ex['start_missed'].append(f'{f}:{a} {k} {name}'); continue
        ck,cn,cv,_,cb,conf=c
        kn = 'macro' if ck=='macrocall' else ck
        if cb!=b: tot['end_mismatch']+=1; ex['end_mismatch'].append(f'{f}:{a} {k} {name} truth_end={b} cut_end={cb} conf={conf}')
        elif kn!=KMAP.get(k,k) and not (k=='macro' and ck in('macro','macrocall')): tot['kind_mismatch']+=1; ex['kind_mismatch'].append(f'{f}:{a} truth={k} cut={ck}')
        elif name!=cn and k not in ('impl','use','macro'): tot['name_mismatch']+=1; ex['name_mismatch'].append(f'{f}:{a} truth={name} cut={cn}')
        elif v!=cv and k not in ('impl','macro','extern_block'): tot['vis_mismatch']+=1; ex['vis_mismatch'].append(f'{f}:{a} {k} {name} truth={v} cut={cv}')
        else: tot['exact']+=1
        if conf<0.5: tot['grey_on_good_code']+=1; ex['grey_on_good_code'].append(f'{f}:{a} {k} {name}')
    for a,c in cmap.items():
        if a not in tmap: tot['extra_cut']+=1; ex['extra_cut'].append(f'{f}:{a} {c[0]} {c[1]}')
print(dict(tot))
for k,v in ex.items(): print(f'-- {k} ({len(v)}):'); [print('   ',x) for x in v[:6]]

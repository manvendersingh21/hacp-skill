#!/usr/bin/env python3
"""Keep command/output transcripts; omit reasoning and unrelated host initialization.
Original stream logs are retained locally outside the publishable repository.
"""
import argparse,hashlib,json,pathlib,shutil
ap=argparse.ArgumentParser();ap.add_argument('directory',type=pathlib.Path);ap.add_argument('--archive',type=pathlib.Path,required=True);args=ap.parse_args()
manifest=[]
for p in sorted(args.directory.rglob('*.jsonl')):
    relative=p.relative_to(args.directory);dest=args.archive/relative
    if dest.exists():raw=dest.read_bytes()
    else:dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,dest);raw=p.read_bytes()
    retained=[]
    for line in raw.decode().splitlines():
        try:d=json.loads(line)
        except ValueError:continue
        typ=d.get('type');event=d.get('event')
        if typ in ('assistant','user'):
            m=d.get('message',{});content=[c for c in m.get('content',[]) if c.get('type') in ('text','tool_use','tool_result')]
            if content:retained.append({'type':typ,'model':m.get('model'),'content':content})
        elif typ in ('item.started','item.completed'):
            if d.get('item',{}).get('type') in ('command_execution','agent_message','file_change','error'):retained.append(d)
        elif typ in ('tool_use','text','error'):
            retained.append(d)
        elif event=='step_update':
            if d.get('step_update',{}).get('step_type') in ('tool','agent_response','error_message'):retained.append(d)
        elif event=='result':
            r=d['result'];retained.append({'event':'result','result':{k:r[k] for k in ('status','response','error','duration_seconds') if k in r}})
    p.write_text(''.join(json.dumps(d,ensure_ascii=False)+'\n' for d in retained))
    manifest.append({'path':str(relative),'original_sha256':hashlib.sha256(raw).hexdigest(),'original_bytes':len(raw),'retained_events':len(retained)})
(args.directory/'transcripts.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(f'Curated {len(manifest)} command/output transcripts; original hashes recorded.')

#!/usr/bin/env python3
"""Opt-in live collaborations. Each CLI gets ONE brief; all peer traffic uses hacp.
Requires authenticated CLIs; acceptance commands and agent edits run in scratch repos.
No models are selected or changed by this harness; the installed defaults are recorded.
"""
import argparse, concurrent.futures, json, os, pathlib, signal, subprocess, tempfile, time
PAIRS=[('claude','codex'),('agy','claude'),('claude','opencode'),('codex','agy'),('opencode','codex'),('agy','opencode')]

def argv(cli,brief,root):
    if cli=='claude':return ['claude','-p','--dangerously-skip-permissions','--output-format','stream-json','--verbose', '/hacp '+brief]
    if cli=='codex':return ['codex','exec','--skip-git-repo-check','--sandbox','workspace-write','--json','-C',str(root),'$hacp '+brief]
    if cli=='agy':return ['agy','--add-dir',str(root),'--dangerously-skip-permissions','--print-timeout','12m','--output-format','stream-json','--print','/hacp '+brief]
    return ['opencode','run','--auto','--format','json','--command','hacp',brief]

def run_pair(pair,dest,timeout):
    name='-'.join(pair);out=dest/name;out.mkdir(parents=True,exist_ok=False)
    root=pathlib.Path(tempfile.mkdtemp(prefix='hacp-live-'+name+'-'))
    subprocess.run(['git','init','-q',str(root)],check=True)
    (root/'.gitignore').write_text('.hacp/\n__pycache__/\n')
    started=time.time();runs=[];handles=[]
    for peer,cli in zip('ab',pair):
        task=('Implement greet.py with greet(name: str) -> str returning Hello, NAME! after stripping surrounding whitespace. Raise ValueError for an empty/whitespace name. Own greet.py.' if peer=='a' else 'Implement test_greet.py with Python unittest tests for greet.greet: normal names, surrounding whitespace, and ValueError for empty/whitespace names. Own test_greet.py.')
        brief=f'''You are peer {peer} in the scratch project {root}. {task}
Your other peer is another independently running CLI. Use the hacp skill and the installed hacp binary. This is a real collaboration: start/join, negotiate one contract for your task, implement after agreement, submit, and cross-verify both tasks. Use python3 -m unittest -v as an acceptance command. You may create your own terms file named terms-{peer}.json as coordination material. Discuss the interface through hacp ask and answer (at least one round trip), and include material behavioral decisions in frozen terms requirements; do not use any other channel to contact your peer. Peer a starts; peer b retries inspection briefly until the session exists then joins. Your peer may take a minute to respond. Keep polling/waiting and stay available after your own task settles until BOTH contracts settle, resolve every outstanding question and run complete (or inspect outcome if the peer completed first). Use close with a reason for intentionally stopped or abandoned work. You are authorized to run the agreed acceptance commands locally in this scratch repository. Do not spawn agents or use brokers, hooks, tmux, or remote services for coordination. Do not read or edit files outside this scratch repository except loading the installed skill. No further user prompts will arrive. If blocked by an operational failure, record the honest outcome. Finish within 10 minutes.'''
        (out/(peer+'-brief.txt')).write_text(brief)
        stdout=open(out/(peer+'.jsonl'),'wb');stderr=open(out/(peer+'.stderr'),'wb');handles.extend([stdout,stderr])
        command=argv(cli,brief,root)
        env=os.environ.copy();env.pop('HACP_PEER',None)
        proc=subprocess.Popen(command,cwd=root,env=env,stdin=subprocess.DEVNULL,stdout=stdout,stderr=stderr,start_new_session=True)
        runs.append((peer,cli,proc));
        if peer=='a':time.sleep(2)
    terminal_events=[];last_events=0
    while any(p.poll() is None for _,_,p in runs) and time.time()-started<timeout:
        snap=root/'.hacp/session.json'
        if snap.exists():
            try:
                data=json.loads(snap.read_text());events=data.get('events',[])
                for e in events[last_events:]:
                    text=f"Peer {e['peer']}: {e['action']}\n"
                    d=e.get('detail',{});body=d.get('body',d)
                    for k in ['text','task','terms','revision','state','next','reason']:
                        if k in body:text+=f"  {k}: {json.dumps(body[k],ensure_ascii=False)}\n"
                    if 'record' in d:text+='  verdict: '+json.dumps(d['record'].get('verdict'))+'\n'
                    terminal_events.append([round(time.time()-started,3),'o',text.replace('\n','\r\n')])
                last_events=len(events)
            except (ValueError,OSError):pass
        time.sleep(.25)
    results=[]
    for peer,cli,p in runs:
        timed_out=p.poll() is None
        if timed_out:os.killpg(p.pid,signal.SIGKILL)
        p.wait();results.append({'peer':peer,'cli':cli,'exit':p.returncode,'timeout':timed_out})
    for f in handles:f.close()
    snapshot=root/'.hacp/session.json';state=json.loads(snapshot.read_text()) if snapshot.exists() else {}
    contracts=state.get('contracts',{});settled=len(contracts)==2 and all(e['contract']['state']=='settled' for e in contracts.values())
    import shutil
    if (root/'.hacp').exists():shutil.copytree(root/'.hacp',out/'state')
    for p in root.iterdir():
        if p.is_file():shutil.copy2(p,out/p.name)
    check=subprocess.run(['python3','-m','unittest','-v'],cwd=root,capture_output=True,text=True)
    (out/'independent-tests.txt').write_text(check.stdout+check.stderr)
    versions={cli:subprocess.check_output([cli,'--version'],text=True).strip() for cli in pair}
    report={'pair':pair,'workspace':str(root),'seconds':round(time.time()-started,2),'processes':results,'versions':versions,'contracts':{k:e['contract']['state'] for k,e in contracts.items()},'session':state.get('session',{}).get('state'),'outcome':state.get('outcome'),'settled':settled,'independent_tests_exit':check.returncode}
    (out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
    (out/'events.cast').write_text(json.dumps({'version':2,'width':120,'height':35,'timestamp':int(started),'title':f'Real HACP collaboration: {name}'})+'\n'+''.join(json.dumps(e)+'\n' for e in terminal_events))
    print(json.dumps(report),flush=True);return report

def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--run-live',action='store_true',required=True);ap.add_argument('--out',type=pathlib.Path,required=True);ap.add_argument('--pair',choices=['-'.join(p) for p in PAIRS]);ap.add_argument('--timeout',type=int,default=720);args=ap.parse_args();args.out.mkdir(parents=True,exist_ok=True)
    pairs=[p for p in PAIRS if not args.pair or '-'.join(p)==args.pair]
    # Two collaborations at a time limits load on each authenticated CLI.
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:results=list(pool.map(lambda p:run_pair(p,args.out,args.timeout),pairs))
    (args.out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
if __name__=='__main__':main()

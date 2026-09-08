#!/usr/bin/env python3
"""Opt-in Codex/AGY small-project collaboration; the harness observes, never acts as a peer."""
import argparse,datetime,hashlib,json,os,pathlib,re,signal,subprocess,time
from live import argv
SPEC='''# Task Pocket: a tiny local task CLI

Build a dependency-free Python 3 task tracker. It must run with the installed python3 (do not assume Python 3.11+).

Public CLI (flags before subcommand):
- python3 task_cli.py --db PATH add "title": print the created task as JSON.
- python3 task_cli.py --db PATH list: print a JSON array of open tasks, sorted by id.
- python3 task_cli.py --db PATH list --all: include completed tasks, sorted by id.
- python3 task_cli.py --db PATH done ID: print the completed task as JSON.

Tasks have integer id, trimmed nonempty string title, and boolean done. First id is 1; IDs increase and are never reused after completion. New tasks start done=false. Repeating done on a completed task succeeds idempotently. Missing task IDs, malformed IDs, and blank titles fail with nonzero exit and a useful stderr message, without corrupting the database or printing a traceback. A missing database lists as []; adding creates its missing parent directories. Data persists across fresh CLI processes. Malformed JSON or an invalid database shape must fail clearly and remain byte-for-byte unchanged. Successful stdout must contain only the specified JSON.

Suggested storage interface to explicitly discuss and agree through hacp:
add_task(path, title) -> task dict; list_tasks(path, include_done=False) -> list of task dicts; complete_task(path, task_id) -> task dict. Use atomic database replacement for writes. No network, dependencies, daemon, or extra features. Concurrent writers are outside this small project's scope.

Peer a (Codex) owns task_store.py: storage, validation, persistence, API.
Peer b (AGY) owns task_cli.py, tests/test_tasks.py, README.md: CLI, unittest coverage, usable README examples.
Both peers may write only their own frozen outputs plus their own terms-a.json/terms-b.json coordination files. SPEC.md is the immutable brief. Do not add unrelated output files.
Acceptance for both contracts must include python3 -m unittest discover -s tests -v and meaningful nonzero test coverage. Wait for the peer's dependencies rather than treating zero discovered tests as completion.
'''

def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--run-live',action='store_true',required=True);ap.add_argument('--project',type=pathlib.Path,required=True);ap.add_argument('--out',type=pathlib.Path,required=True);ap.add_argument('--timeout',type=int,default=900);a=ap.parse_args()
    root=a.project.resolve();out=a.out.resolve();root.mkdir(parents=True,exist_ok=False);out.mkdir(parents=True,exist_ok=False)
    subprocess.run(['git','init','-q',str(root)],check=True);(root/'SPEC.md').write_text(SPEC);(root/'.gitignore').write_text('.hacp/\n__pycache__/\n*.pyc\ntasks.json\n')
    (out/'SPEC.md').write_text(SPEC)
    installed=pathlib.Path(subprocess.check_output(['which','hacp'],text=True).strip())
    conf=pathlib.Path.home()/'.codex/config.toml';match=re.search(r'^model\s*=\s*"([^"]+)"',conf.read_text(),re.M)
    models={'codex':match.group(1) if match else None,'agy':json.loads((pathlib.Path.home()/'.gemini/antigravity-cli/settings.json').read_text()).get('model')}
    environment={'project':str(root),'started_utc':datetime.datetime.utcnow().isoformat()+'Z','models':models,'versions':{c:subprocess.check_output([c,'--version'],text=True).strip() for c in ['hacp','codex','agy']},'binary_sha256':hashlib.sha256(installed.read_bytes()).hexdigest()}
    (out/'environment.json').write_text(json.dumps(environment,indent=2)+'\n');print(json.dumps(environment),flush=True)
    started=time.monotonic();runs=[];streams=[];observations=[];seen=set();last_events=0
    ownership={'a':['task_store.py'],'b':['task_cli.py','tests/test_tasks.py','README.md']}
    for peer,cli in [('a','codex'),('b','agy')]:
        brief=f'''You are peer {peer} in the scratch project {root}. Read SPEC.md for the complete small-project requirements. Your CLI is {cli}; your owned output files are {', '.join(ownership[peer])}. Use the installed hacp skill and binary for a real bilateral collaboration with the other independently running CLI. Peer a starts and peer b joins. Before editing implementation, tests, or README, discuss the suggested interface through hacp ask/answer and explicitly freeze BOTH contracts. Negotiate one contract for your own task, using your own terms-{peer}.json. Include all your files in its outputs. Use python3 -m unittest discover -s tests -v for acceptance, and require meaningful tests (zero tests is not success). Work only on your frozen outputs. Poll before edits and after meaningful tests. Any necessary interface/output changes require an amendment. Report actual failures honestly and request/perform rework as needed. Submit your complete outputs, verify the other peer's submission, and remain available until both contracts settle. Answer ALL outstanding questions before final closure. Close only after both contracts settle, or explicitly explain unresolved failure. You may edit this scratch project and run agreed local acceptance commands. Coordinate ONLY through hacp after this initial brief; never write directly to .hacp, send external messages, or spawn other agents. Do not change installed host configuration. Use the configured model. No further user prompts will arrive. Aim to finish in 10 minutes.'''
        (out/(peer+'-brief.txt')).write_text(brief)
        stdout=open(out/(peer+'.jsonl'),'wb');stderr=open(out/(peer+'.stderr'),'wb');streams.extend([stdout,stderr]);env=os.environ.copy();env.pop('HACP_PEER',None)
        command=argv(cli,brief,str(root));p=subprocess.Popen(command,cwd=root,env=env,stdin=subprocess.DEVNULL,stdout=stdout,stderr=stderr,start_new_session=True);runs.append((peer,cli,p))
        if peer=='a':time.sleep(2)
    while any(p.poll() is None for _,_,p in runs) and time.monotonic()-started<a.timeout:
        path=root/'.hacp/session.json';s=None
        if path.exists():
            try:s=json.loads(path.read_text())
            except (ValueError,OSError):pass
        if s:
            events=s.get('events',[])
            for e in events[last_events:]:observations.append({'seconds':round(time.monotonic()-started,3),'event_id':e['id'],'peer':e['peer'],'action':e['action']})
            last_events=len(events)
        for peer,files in ownership.items():
            for file in files:
                if file in seen or not (root/file).exists():continue
                seen.add(file);contracts=list((s or {}).get('contracts',{}).values());frozen=[e for e in contracts if e['contract']['revisions']]
                observations.append({'seconds':round(time.monotonic()-started,3),'first_file_seen':file,'owner':peer,'both_contracts_frozen':len(frozen)==2,'owner_contract_frozen':any(e['contract']['task']['owner']=='urn:hacp:agent:'+peer and file in e['contract']['revisions'][-1]['content']['outputs'] for e in frozen)})
        time.sleep(.1)
    processes=[]
    for peer,cli,p in runs:
        timed_out=p.poll() is None
        if timed_out:os.killpg(p.pid,signal.SIGKILL)
        p.wait();processes.append({'peer':peer,'cli':cli,'exit':p.returncode,'timeout':timed_out})
    for f in streams:f.close()
    (out/'observations.json').write_text(json.dumps(observations,indent=2)+'\n')
    import shutil
    if (root/'.hacp').exists():shutil.copytree(root/'.hacp',out/'state')
    for file in ['SPEC.md']+ownership['a']+ownership['b']:
        p=root/file
        if p.exists():dest=out/'project'/file;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,dest)
    state=json.loads((root/'.hacp/session.json').read_text()) if (root/'.hacp/session.json').exists() else {}
    cs=state.get('contracts',{});settled=len(cs)==2 and all(e['contract']['state']=='settled' for e in cs.values())
    check=subprocess.run(['python3','-m','unittest','discover','-s','tests','-v'],cwd=root,capture_output=True,text=True)
    (out/'peer-tests.txt').write_text(check.stdout+check.stderr)
    result={'seconds':round(time.monotonic()-started,2),'project':str(root),'processes':processes,'session':state.get('session',{}).get('state'),'contracts':{k:e['contract']['state'] for k,e in cs.items()},'settled':settled,'peer_tests_exit':check.returncode}
    (out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result),flush=True)
if __name__=='__main__':main()

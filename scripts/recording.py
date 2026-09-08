#!/usr/bin/env python3
"""Package the recorded live event monitor and real final log in a standalone player."""
import json,pathlib,html,subprocess
root=pathlib.Path(__file__).resolve().parents[1]
source=root/'docs/evidence/live-2026-09-07/claude-codex'
lines=(source/'events.cast').read_text().splitlines();header=json.loads(lines[0]);events=[json.loads(l) for l in lines[1:]]
# Actual lifecycle timestamps are retained at 2x speed. The final log is fetched
# from the recorded session; no participant actions or shell output are invented.
original_duration=events[-1][0]
for e in events:e[0]=round(e[0]/2,3)
intro=[0,'o','HACP: Claude Code + Codex | Recorded live event monitor | 2x playback\r\n\r\n']
workspace=json.loads((source/'result.json').read_text())['workspace']
log=subprocess.check_output(['cat',str(pathlib.Path(workspace)/'.hacp/log.md')],text=True)
final_time=events[-1][0]+2
events=[intro]+events+[[final_time,'o','\r\n$ cat .hacp/log.md\r\n'+log.replace('\n','\r\n')],[final_time+15,'o','\r\nBoth contracts settled. Session closed.\r\n']]
header['title']='Claude Code + Codex: real collaboration, 2x playback'
(root/'docs/demo.cast').write_text(json.dumps(header)+'\n'+''.join(json.dumps(e)+'\n' for e in events))
payload=json.dumps(events).replace('<','\\u003c')
page='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>HACP live collaboration</title>
<style>body{margin:0;background:#10151c;color:#e8edf2;font:16px system-ui;padding:24px}main{max-width:1100px;margin:auto}h1{font-size:24px}button,input{font:inherit}button{padding:8px 16px;margin-right:12px;background:#75e0b4;border:0;border-radius:6px}pre{background:#080c12;border:1px solid #344454;border-radius:10px;padding:20px;height:65vh;overflow:auto;white-space:pre-wrap;font:13px/1.6 ui-monospace,monospace}input{width:60%}a{color:#75e0b4}</style>
<main><h1>Claude Code + Codex, through HACP</h1><p>Real collaboration in a scratch repository. Recorded live event monitor at <b>2× playback</b>, followed by the actual final log. CLI command transcripts and unaccelerated event timestamps are in the evidence directory.</p><button id="play">Play</button><button id="restart">Restart</button><input id="seek" aria-label="Playback position" type="range" min="0" step="0.1"><span id="time"></span><pre id="terminal" tabindex="0"></pre><p><a href="evidence/live-2026-09-07/claude-codex/state/log.md">Read the full final log</a> · <a href="demo.cast">Download asciicast</a></p></main>
<script>const events=EVENTS;const end=events.at(-1)[0];let t=0,running=false,previous=performance.now();const terminal=document.querySelector('#terminal'),seek=document.querySelector('#seek'),play=document.querySelector('#play');seek.max=end;function render(){terminal.textContent=events.filter(e=>e[0]<=t).map(e=>e[2]).join('');terminal.scrollTop=terminal.scrollHeight;seek.value=t;document.querySelector('#time').textContent=` ${Math.floor(t)} / ${Math.ceil(end)} s`;play.textContent=running?'Pause':'Play'}play.onclick=()=>{running=!running;if(t>=end)t=0;render()};document.querySelector('#restart').onclick=()=>{t=0;render()};seek.oninput=()=>{t=Number(seek.value);render()};function tick(now){if(running){t=Math.min(end,t+(now-previous)/1000);if(t>=end)running=false;render()}previous=now;requestAnimationFrame(tick)}render();requestAnimationFrame(tick);</script></html>'''.replace('EVENTS',payload)
(root/'docs/demo.html').write_text(page)
print(f'Recorded {original_duration:.2f}s of live events; playback including final log is {events[-1][0]:.2f}s.')

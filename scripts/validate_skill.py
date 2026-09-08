#!/usr/bin/env python3
"""Dependency-free validation of this package's shared skill and embedded wrapper."""
from pathlib import Path
root=Path(__file__).resolve().parents[1]
text=(root/'skills/hacp/SKILL.md').read_text()
assert text.startswith('---\n')
front,body=text[4:].split('\n---\n',1)
fields=dict(line.split(':',1) for line in front.splitlines())
assert fields['name'].strip()=='hacp'
assert fields['description'].strip()
assert body.strip() and len(text.split())<1000
assert 'TODO' not in text and '[INSERT' not in text
print('Shared hacp skill structure valid.')

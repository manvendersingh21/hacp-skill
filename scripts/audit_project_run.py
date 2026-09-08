#!/usr/bin/env python3
"""Audit a completed Task Pocket live run without acting as either HACP peer."""
import argparse
import hashlib
import json
import pathlib
import re
import subprocess
import sys


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('evidence', type=pathlib.Path)
    args = parser.parse_args()
    out = args.evidence.resolve()
    result = json.loads((out / 'result.json').read_text())
    project = pathlib.Path(result['project'])
    state = json.loads((out / 'state/session.json').read_text())
    contracts = list(state['contracts'].values())
    observations = json.loads((out / 'observations.json').read_text())
    files = [o for o in observations if 'first_file_seen' in o]
    questions = [m for m in state['messages'] if m['kind'] == 'hacp.skill.ask']
    answers = [m for m in state['messages'] if m['kind'] == 'hacp.skill.answer']
    answered = {m.get('in_reply_to') for m in answers}
    artifacts = []
    for entry in contracts:
        latest = set(entry['submissions'][-1]['artifacts']) if entry['submissions'] else set()
        for artifact in entry['artifacts']:
            record = artifact['record']
            copy = out / 'state' / pathlib.Path(record['location']).relative_to('.hacp')
            working = project / artifact['path']
            current = record['artifact_id'] in latest
            artifacts.append({
                'path': artifact['path'],
                'artifact_id': record['artifact_id'],
                'latest_submission': current,
                'copy_hash_matches': digest(copy) == record['digest'],
                'copy_size_matches': copy.is_file() and copy.stat().st_size == record['size'],
                'working_hash_matches': digest(working) == record['digest'] if current else None,
                'working_size_matches': working.is_file() and working.stat().st_size == record['size'] if current else None,
                'revision_exists': any(r['digest'] == record['contract_revision'] for r in entry['contract']['revisions']),
            })
    suite = subprocess.run(
        [sys.executable, str(pathlib.Path(__file__).with_name('audit_task_project.py')), str(project)],
        capture_output=True, text=True, timeout=180,
    )
    (out / 'independent-tests.txt').write_text(suite.stdout + suite.stderr)
    peer_output = (out / 'peer-tests.txt').read_text()
    count = re.search(r'Ran (\d+) tests?', peer_output)
    checks = {
        'both_processes_successful': len(result['processes']) == 2 and all(p['exit'] == 0 and not p['timeout'] for p in result['processes']),
        'both_contracts_settled': len(contracts) == 2 and all(e['contract']['state'] == 'settled' for e in contracts),
        'session_closed': state['session']['state'] == 'closed',
        'all_questions_answered': all(q['message_id'] in answered for q in questions),
        'bilateral_question_roundtrip': any(a.get('in_reply_to') == q['message_id'] and a['from'] == q['to'] and a['to'] == q['from'] for q in questions for a in answers),
        'cross_verification': len(contracts) == 2 and all(e['verifications'] and e['verifications'][-1]['verdict'] == 'accept' and all(v['verifier'] != e['contract']['task']['owner'] and v['verifier'] in e['contract']['participants'] for v in e['verifications']) for e in contracts),
        'all_output_files_after_both_freezes': bool(files) and all(o['both_contracts_frozen'] and o['owner_contract_frozen'] for o in files),
        'observed_all_four_outputs': {o['first_file_seen'] for o in files} == {'task_store.py', 'task_cli.py', 'tests/test_tasks.py', 'README.md'},
        'all_artifacts_intact': len(artifacts) >= 4 and all(a['copy_hash_matches'] and a['copy_size_matches'] and a['revision_exists'] and (not a['latest_submission'] or (a['working_hash_matches'] and a['working_size_matches'])) for a in artifacts),
        'spec_unchanged': (out / 'SPEC.md').read_bytes() == (project / 'SPEC.md').read_bytes(),
        'peer_tests_passed_nonzero': result['peer_tests_exit'] == 0 and count is not None and int(count[1]) > 0,
        'independent_12_tests_passed': suite.returncode == 0 and 'Ran 12 tests' in suite.stderr,
    }
    report = {
        'checks': checks, 'passed': all(checks.values()),
        'peer_test_count': int(count[1]) if count else 0,
        'question_count': len(questions), 'answer_count': len(answers),
        'artifact_count': len(artifacts), 'artifacts': artifacts, 'file_observations': files,
        'verification_verdicts': [v['verdict'] for e in contracts for v in e['verifications']],
    }
    (out / 'audit.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'evidence': str(out), **report}, indent=2))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())

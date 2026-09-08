#!/usr/bin/env python3
"""Independent black-box checks for the SPEC.md CLI, written outside the peers' output files."""
import argparse,json,pathlib,subprocess,tempfile,unittest,sys
PROJECT=None
class TaskCLI(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='hacp-blackbox-');self.addCleanup(self.temp.cleanup);self.db=pathlib.Path(self.temp.name)/'tasks.json'
    def call(self,*args,success=True):
        p=subprocess.run([sys.executable,str(PROJECT/'task_cli.py'),'--db',str(self.db),*map(str,args)],cwd=PROJECT,capture_output=True,text=True,timeout=10)
        if success:
            self.assertEqual(p.returncode,0,p.stderr);self.assertEqual(p.stderr,'',p.stderr)
            try:return json.loads(p.stdout)
            except ValueError:self.fail('stdout is not one JSON value: '+p.stdout)
        self.assertNotEqual(p.returncode,0);self.assertTrue(p.stderr.strip());self.assertNotIn('Traceback',p.stderr);self.assertEqual(p.stdout,'');return p
    def seed(self):return self.call('add','First task')
    def test_missing_database_lists_empty(self):self.assertEqual(self.call('list'),[])
    def test_add_trims_title_and_returns_task(self):self.assertEqual(self.call('add','  Write tests  '),{'id':1,'title':'Write tests','done':False})
    def test_unicode_and_quotes_roundtrip(self):
        title='Read "café" ☕';self.assertEqual(self.call('add',title)['title'],title);self.assertEqual(self.call('list')[0]['title'],title)
    def test_nested_parent_directories(self):
        self.db=self.db.parent/'new'/'nested'/'tasks.json';self.assertEqual(self.seed()['id'],1);self.assertTrue(self.db.is_file());self.assertEqual(len(self.call('list')),1)
    def test_persistence_filtering_and_sorted_ids(self):
        self.call('add','One');self.call('add','Two');self.call('done',1)
        self.assertEqual(self.call('list'),[{'id':2,'title':'Two','done':False}]);self.assertEqual(self.call('list','--all'),[{'id':1,'title':'One','done':True},{'id':2,'title':'Two','done':False}])
    def test_completion_is_idempotent(self):
        self.seed();first=self.call('done',1);self.assertTrue(first['done']);self.assertEqual(self.call('done',1),first)
    def test_ids_not_reused_after_all_completed(self):
        self.seed();self.call('done',1);self.assertEqual(self.call('add','Next')['id'],2)
    def test_blank_titles_refused_without_mutation(self):
        self.seed();before=self.db.read_bytes()
        for title in ['', ' \t\n ']:
            with self.subTest(title=title):self.call('add',title,success=False);self.assertEqual(self.db.read_bytes(),before)
    def test_unknown_task_refused_without_mutation(self):
        self.seed();before=self.db.read_bytes();self.call('done',999,success=False);self.assertEqual(self.db.read_bytes(),before)
    def test_malformed_ids_refused_without_mutation(self):
        self.seed();before=self.db.read_bytes()
        for id in ['abc','1.5','0','-1']:
            with self.subTest(id=id):self.call('done',id,success=False);self.assertEqual(self.db.read_bytes(),before)
    def test_corrupt_json_never_overwritten(self):
        bad=b'{not JSON\n';self.db.write_bytes(bad)
        for args in [('list',),('add','Task'),('done','1')]:
            with self.subTest(args=args):self.call(*args,success=False);self.assertEqual(self.db.read_bytes(),bad)
    def test_invalid_database_shape_never_overwritten(self):
        for value in [None,42,'bad',{'tasks':'bad'}, {'tasks':[{'id':'invalid','title':None,'done':'maybe'}]}]:
            bad=json.dumps(value).encode();self.db.write_bytes(bad)
            for args in [('list',),('add','Task'),('done','1')]:
                with self.subTest(value=value,args=args):self.call(*args,success=False);self.assertEqual(self.db.read_bytes(),bad)
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('project',type=pathlib.Path);args=p.parse_args();PROJECT=args.project.resolve()
    result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(TaskCLI));sys.exit(not result.wasSuccessful())

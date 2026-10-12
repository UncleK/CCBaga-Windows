import importlib.util
import json
import tempfile
import unittest
import zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('fetch_windows',ROOT/'scripts/fetch-browser-windows.py')
engine=importlib.util.module_from_spec(spec)
spec.loader.exec_module(engine)

class WindowsPackaging(unittest.TestCase):
    def test_rejects_unsafe_zip(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            for name in ['../outside','C:/outside','dir/CON.txt','dir/data:stream','dir/trailing.','dir\\..\\outside']:
                with self.subTest(name=name):
                    archive=root/'input.zip'
                    with zipfile.ZipFile(archive,'w') as z:z.writestr(name,'unsafe')
                    with self.assertRaises(ValueError):engine.extract(archive,root/'output')
    def test_unicode_zip(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);archive=root/'input.zip'
            with zipfile.ZipFile(archive,'w') as z:z.writestr('测试 folder/data.txt','ok')
            engine.extract(archive,root/'output')
            self.assertEqual((root/'output/测试 folder/data.txt').read_text(),'ok')
    def test_windows_packaging_is_independent(self):
        config=json.loads((ROOT/'src-tauri/tauri.windows.conf.json').read_text())
        self.assertEqual(config['bundle']['windows']['nsis']['installMode'],'currentUser')
        self.assertEqual(config['plugins']['updater']['endpoints'],[])
        self.assertNotIn('sh ',config['build']['beforeBuildCommand'])

if __name__=='__main__':unittest.main()

"""Package layout tests use synthetic executable headers, not runnable cross-platform builds."""
from pathlib import Path
import tempfile, unittest, zipfile
from package import stage_release
from verify_release import inspect

def executable(system):
    data=bytearray(256)
    if system=='Linux':data[:6]=b'\x7fELF\x02\x01';data[18:20]=(62).to_bytes(2,'little')
    elif system=='Windows':data[:2]=b'MZ';data[60:64]=(128).to_bytes(4,'little');data[128:134]=b'PE\0\0\x64\x86'
    else:data[:8]=b'\xcf\xfa\xed\xfe'+(0x100000c).to_bytes(4,'little')
    return data

def prepare(folder,system):
    root=folder/'source';(root/'target/release').mkdir(parents=True)
    suffix='.exe' if system=='Windows' else ''
    for name in ['tinkercraft','tinkercraft-cli']:(root/'target/release'/(name+suffix)).write_bytes(executable(system))
    for name in ['examples','dependency-licenses']:(root/name).mkdir();(root/name/'fixture.txt').write_text('fixture')
    (root/'dependency-licenses.csv').write_text('crate,license\n')
    (root/'README.md').write_text('test fixture')
    return root

def archive(stage,path):
    with zipfile.ZipFile(path,'w') as z:
        for p in stage.rglob('*'):
            if p.is_file():z.write(p,stage.name+'/'+p.relative_to(stage).as_posix())

class Packaging(unittest.TestCase):
    def test_three_platform_layouts(self):
        for system,arch in [('Linux','x86_64'),('Windows','x86_64'),('Darwin','aarch64')]:
            with self.subTest(system=system),tempfile.TemporaryDirectory() as directory:
                folder=Path(directory);root=prepare(folder,system);stage=folder/'package'
                stage_release(root,stage,system,arch,'0.11.0',{'synthetic_fixture':True})
                path=folder/'package.zip';archive(stage,path);m,_,_=inspect(path)
                self.assertEqual((m['platform'],m['arch']),(system,arch))
                self.assertTrue((stage/m['desktop']).is_file())
    def test_wrong_native_binary_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            folder=Path(directory);root=prepare(folder,'Linux')
            with self.assertRaises(ValueError):stage_release(root,folder/'package','Darwin','aarch64','0.11.0',{})
    def test_tampering_detected(self):
        with tempfile.TemporaryDirectory() as directory:
            folder=Path(directory);root=prepare(folder,'Linux');stage=folder/'package'
            stage_release(root,stage,'Linux','x86_64','0.11.0',{})
            (stage/'README.md').write_text('modified')
            path=folder/'package.zip';archive(stage,path)
            with self.assertRaisesRegex(ValueError,'Checksum mismatch'):inspect(path)
    def test_unsafe_archive_path_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'bad.zip'
            with zipfile.ZipFile(path,'w') as z:z.writestr('../outside','x')
            with self.assertRaisesRegex(ValueError,'Unsafe'):inspect(path)
if __name__=='__main__':unittest.main()

"""Verify package contents; --run additionally executes its native CLI and version probe."""
from pathlib import Path, PurePosixPath
import argparse, hashlib, json, os, platform, plistlib, subprocess, tarfile, tempfile, zipfile
from package import binary_target, architecture

def inspect(archive):
    contents={}
    def add(name,data):
        path=PurePosixPath(name)
        if path.is_absolute() or '..' in path.parts or '\\' in name or ':' in name:raise ValueError('Unsafe package path')
        if name in contents:raise ValueError('Duplicate package entry')
        contents[name]=data
    if zipfile.is_zipfile(archive):
        with zipfile.ZipFile(archive) as z:
            for i in z.infolist():
                if i.is_dir():continue
                if (i.external_attr>>16)&0o170000==0o120000:raise ValueError('Unexpected symlink')
                add(i.filename,z.read(i))
    else:
        with tarfile.open(archive) as t:
            for i in t.getmembers():
                if i.isdir():continue
                if not i.isfile():raise ValueError('Unexpected special file')
                add(i.name,t.extractfile(i).read())
    manifests=[name for name in contents if name.endswith('/release.json')]
    if len(manifests)!=1:raise ValueError('Expected one release manifest')
    name=manifests[0];prefix=name[:-len('release.json')];m=json.loads(contents[name])
    expected={prefix+p for p in m['files']}|{name}
    if expected!=set(contents):raise ValueError('Package file inventory mismatch')
    for p,digest in m['files'].items():
        if hashlib.sha256(contents[prefix+p]).hexdigest()!=digest:raise ValueError('Checksum mismatch: '+p)
    for key in ('desktop','cli'):
        if binary_target(contents[prefix+m[key]])!=(m['platform'],m['arch']):raise ValueError('Executable target mismatch')
    if m['platform']=='Darwin':
        plist=plistlib.loads(contents[prefix+'TinkerCraft.app/Contents/Info.plist'])
        if plist['CFBundleShortVersionString']!=m['version'] or plist['CFBundleExecutable']!='tinkercraft':raise ValueError('Mac bundle metadata mismatch')
        if plist.get('CFBundleIconFile')!='TinkerCraft.icns' or prefix+'TinkerCraft.app/Contents/Resources/TinkerCraft.icns' not in contents:raise ValueError('Mac icon missing')
    return m,prefix,contents

def main():
    parser=argparse.ArgumentParser();parser.add_argument('archive',type=Path);parser.add_argument('--run',action='store_true');parser.add_argument('--window-smoke',action='store_true');args=parser.parse_args()
    if args.window_smoke and not args.run:parser.error('--window-smoke requires --run')
    m,prefix,contents=inspect(args.archive)
    if args.run:
        if (m['platform'],m['arch'])!=(platform.system(),architecture()):raise ValueError('Run this package on its target OS/architecture')
        with tempfile.TemporaryDirectory(prefix='tinkercraft-release-') as folder:
            folder=Path(folder)
            for name,data in contents.items():
                p=folder.joinpath(*PurePosixPath(name).parts);p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data)
            cli=folder/(prefix+m['cli']);app=folder/(prefix+m['desktop']);cli.chmod(0o755);app.chmod(0o755)
            if m['platform']=='Darwin':
                subprocess.run(['codesign','--verify','--deep','--strict','--verbose=2',str(folder/(prefix+'TinkerCraft.app'))],check=True)
            # The runtime probe uses the extracted package and no repository working directory.
            env=os.environ.copy();env.pop('CARGO_MANIFEST_DIR',None)
            probe=json.loads(subprocess.check_output([str(cli),'self-test'],cwd=folder,env=env,text=True))
            if probe.get('self_test')!='passed' or probe.get('version')!=m['version']:raise ValueError('Packaged CLI self-test failed')
            if subprocess.check_output([str(app),'--version'],cwd=folder,env=env,text=True).strip()!=f"TinkerCraft {m['version']}":raise ValueError('Packaged desktop version mismatch')
            if args.window_smoke:
                report=folder/'native-window.json'
                result=subprocess.run([str(app),'--smoke-test',str(report)],cwd=folder,env=env,timeout=90)
                if result.returncode:
                    raise RuntimeError(report.read_text() if report.exists() else f'Native app exited {result.returncode} without a report')
                evidence=json.loads(report.read_text())
                if evidence.get('native_window')!='passed' or evidence.get('version')!=m['version']:raise ValueError('Packaged native window failed')
                print(json.dumps(evidence))
    print(json.dumps({'package':'verified','version':m['version'],'platform':m['platform'],'arch':m['arch'],'files':len(m['files']),'native_cli_run':args.run}))
if __name__=='__main__':main()

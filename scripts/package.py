"""Package and identify a native build. Run on the target OS after cargo build --release."""
from pathlib import Path
import hashlib, json, platform, plistlib, re, shutil, subprocess, tarfile, zipfile

def binary_target(data):
    if data[:4]==b'\x7fELF' and len(data)>=20:
        machine=int.from_bytes(data[18:20],'little' if data[5]==1 else 'big')
        return 'Linux',{62:'x86_64',183:'aarch64'}.get(machine,'unknown')
    if data[:2]==b'MZ' and len(data)>=64:
        pe=int.from_bytes(data[60:64],'little')
        if data[pe:pe+4]==b'PE\0\0':return 'Windows',{0x8664:'x86_64',0xaa64:'aarch64'}.get(int.from_bytes(data[pe+4:pe+6],'little'),'unknown')
    if data[:4]==b'\xcf\xfa\xed\xfe' and len(data)>=8:
        return 'Darwin',{0x1000007:'x86_64',0x100000c:'aarch64'}.get(int.from_bytes(data[4:8],'little'),'unknown')
    raise ValueError('Unrecognized or unsupported native executable')

def architecture():
    return {'AMD64':'x86_64','arm64':'aarch64'}.get(platform.machine(),platform.machine())

def stage_release(root,stage,system,arch,version,evidence):
    suffix='.exe' if system=='Windows' else ''
    sources=[root/'target/release'/('tinkercraft'+suffix),root/'target/release'/('tinkercraft-cli'+suffix)]
    for path in sources:
        if binary_target(path.read_bytes())!=(system,arch):raise ValueError(f'Wrong executable platform/architecture: {path.name}')
    stage.mkdir(parents=True)
    if system=='Darwin':
        contents=stage/'TinkerCraft.app/Contents';(contents/'MacOS').mkdir(parents=True)
        app=contents/'MacOS/tinkercraft';shutil.copy2(sources[0],app)
        with (contents/'Info.plist').open('wb') as f:
            plistlib.dump({'CFBundleName':'TinkerCraft','CFBundleDisplayName':'TinkerCraft',
                          'CFBundleIdentifier':'com.tinkercraft.native','CFBundleExecutable':'tinkercraft',
                          'CFBundlePackageType':'APPL','CFBundleShortVersionString':version,
                          'CFBundleVersion':version,'NSHighResolutionCapable':True,
                          'NSHumanReadableCopyright':'TinkerCraft contributors'},f)
    else:
        app=stage/sources[0].name;shutil.copy2(sources[0],app)
    cli=stage/sources[1].name;shutil.copy2(sources[1],cli)
    if system!='Windows':app.chmod(0o755);cli.chmod(0o755)
    for name in ('README.md','LICENSE','THIRD_PARTY.md','MIGRATION.md','ROADMAP.md','CHECKPOINTS.md','CHECKPOINT10.md','RELEASE.md'):
        if (root/name).exists():shutil.copy2(root/name,stage/name)
    for name in ('examples','dependency-licenses'):shutil.copytree(root/name,stage/name)
    shutil.copy2(root/'dependency-licenses.csv',stage/'dependency-licenses.csv')
    files={p.relative_to(stage).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(stage.rglob('*')) if p.is_file()}
    manifest={'name':'TinkerCraft','version':version,'platform':system,'arch':arch,
              'desktop':app.relative_to(stage).as_posix(),'cli':cli.relative_to(stage).as_posix(),
              'native_cli_self_test':evidence,'gui_runtime':'not checked by packager; see RELEASE.md',
              'files':files}
    (stage/'release.json').write_text(json.dumps(manifest,indent=2)+'\n')
    return manifest

def main():
    root=Path(__file__).resolve().parents[1];out=root/'dist';out.mkdir(exist_ok=True)
    system=platform.system();arch=architecture()
    version=re.search(r'^version = "([^"]+)"',(root/'Cargo.toml').read_text(),re.M)[1]
    suffix='.exe' if system=='Windows' else ''
    cli=root/'target/release'/('tinkercraft-cli'+suffix)
    app=root/'target/release'/('tinkercraft'+suffix)
    evidence=json.loads(subprocess.check_output([str(cli),'self-test'],text=True))
    if evidence.get('self_test')!='passed' or evidence.get('version')!=version:raise ValueError('Build version/self-test mismatch')
    if subprocess.check_output([str(app),'--version'],text=True).strip()!=f'TinkerCraft {version}':raise ValueError('Desktop version mismatch')
    stage=out/f'TinkerCraft-{system}-{arch}'
    if stage.exists():shutil.rmtree(stage)
    stage_release(root,stage,system,arch,version,evidence)
    if system=='Linux':
        archive=stage.with_suffix('.tar.gz')
        with tarfile.open(archive,'w:gz') as t:t.add(stage,arcname=stage.name)
    else:
        archive=stage.with_suffix('.zip')
        with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
            for p in sorted(stage.rglob('*')):
                if p.is_file():z.write(p,p.relative_to(out))
    shutil.rmtree(stage)
    archive.with_name(archive.name+'.sha256').write_text(hashlib.sha256(archive.read_bytes()).hexdigest()+'  '+archive.name+'\n')
    print(archive)
if __name__=='__main__':main()

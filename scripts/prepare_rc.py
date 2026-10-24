"""Stamp a CI build as a release candidate before compiling or packaging.
The workflow run number is stable across partial reruns, so all OS assets agree.
"""
from pathlib import Path
import os,re,tomllib


def prepare(root,number):
    if not re.fullmatch(r'[1-9][0-9]*',number):
        raise ValueError('RC number must be a positive integer')
    manifest=root/'Cargo.toml'
    source=manifest.read_text()
    old=tomllib.loads(source)['workspace']['package']['version']
    base=old.split('-',1)[0]
    if not re.fullmatch(r'\d+\.\d+\.\d+',base):raise ValueError('Expected semantic base version')
    version=f'{base}-rc.{number}'
    lock=root/'Cargo.lock';text=lock.read_text()
    pattern=r'(name = "tinkercraft(?:-cli|-core)?"\nversion = )"[^"]+"'
    text,count=re.subn(pattern,lambda m:m[1]+f'"{version}"',text)
    if count!=3:raise ValueError('Expected exactly three workspace packages in Cargo.lock')
    source,count=re.subn(r'^version = "'+re.escape(old)+r'"$',f'version = "{version}"',source,count=1,flags=re.M)
    if count!=1:raise ValueError('Workspace version not found')
    manifest.write_text(source);lock.write_text(text)
    return version


if __name__=='__main__':
    print(prepare(Path(__file__).resolve().parents[1],os.environ['GITHUB_RUN_NUMBER']))

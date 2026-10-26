"""Exercise native SVG/font CLI conversion and colored 3MF export on each platform."""
import json, os, subprocess, sys, tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[1]
cli=root/'target/release'/('tinkercraft-cli.exe' if os.name=='nt' else 'tinkercraft-cli')
metadata=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--locked'],cwd=root,text=True))
package=next(p for p in metadata['packages'] if p['name']=='epaint_default_fonts')
font=Path(package['manifest_path']).parent/'fonts/Ubuntu-Light.ttf'
output=Path(sys.argv[1] if len(sys.argv)>1 else 'outline-test.3mf').resolve()
with tempfile.TemporaryDirectory(prefix='tinkercraft-outlines-') as temp:
    temp=Path(temp);svg=temp/'svg.tinkercraft';letters=temp/'font.tinkercraft'
    subprocess.run([str(cli),'import-svg',str(root/'examples/outline-badge.svg'),str(svg),'60','3'],check=True)
    subprocess.run([str(cli),'font-text',str(font),str(letters),'60','2','BOLT 12'],check=True)
    p=json.loads(svg.read_text());n=json.loads(letters.read_text())['parts'][0]
    n['position'][0]+=100;n['color']=[220,160,80];p['parts'].append(n)
    combined=temp/'combined.tinkercraft';combined.write_text(json.dumps(p))
    subprocess.run([str(cli),'validate',str(combined)],check=True)
    subprocess.run([str(cli),'export-3mf',str(combined),str(output)],check=True)
print(output)

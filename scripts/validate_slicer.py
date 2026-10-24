"""Check real Bambu Studio imports against the two-color/two-hole fixture.
Usage: python scripts/validate_slicer.py /path/to/bambu-studio
The caller supplies a native Bambu executable and its runtime dependencies.
This imports models only; it does not slice G-code or operate a printer.
"""
import argparse,json,re,subprocess,tempfile
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('bambu');a=p.parse_args()
root=Path(__file__).resolve().parents[1]
cli=root/'target/release/tinkercraft-cli'
if not cli.exists():cli=cli.with_suffix('.exe')
with tempfile.TemporaryDirectory(prefix='tinkercraft-slicer-') as folder:
    folder=Path(folder);results={}
    for kind,command,parts in [('stl','export',1),('3mf','export-3mf',2)]:
        model=folder/('plate.'+kind)
        subprocess.run([str(cli),command,str(root/'examples/multicolor-plate.tinkercraft'),str(model)],check=True)
        result=subprocess.run([a.bambu,'--datadir',str(folder/'settings'),'--info',str(model)],cwd=folder,capture_output=True,text=True,timeout=90,check=True)
        fields=dict(re.findall(r'^(\w+)\s*=\s*(.+)$',result.stdout,re.M))
        for axis,expected in [('x',80),('y',30),('z',5)]:
            assert abs(float(fields['size_'+axis])-expected)<1e-4,result.stdout
        assert fields['manifold'].strip()=='yes',result.stdout
        assert int(fields['number_of_parts'])==parts,result.stdout
        assert abs(float(fields['volume'])-11803.790612309556)<.02,result.stdout
        results[kind]=fields
    print(json.dumps({'slicer':'Bambu Studio','imports':results,'slicing_or_printing_tested':False},indent=2))

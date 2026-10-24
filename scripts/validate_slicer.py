"""Validate real Bambu imports; optionally slice with bundled X1C/PLA profiles.
Usage: python scripts/validate_slicer.py /path/to/bambu-studio \
    --profiles /path/to/resources/profiles/BBL --report validation.json
All generated G-code stays in a temporary directory and is deleted, never sent
or offered to a printer. Profile choice is a regression fixture, not a statement
about the user's nozzle, plate, filament or calibrated printer settings.
"""
import argparse,hashlib,json,re,subprocess,tempfile,xml.etree.ElementTree as ET,zipfile
from pathlib import Path


def resolve_profile(name,paths,stack=()):
    if name in stack:raise ValueError('Cyclic profile inheritance: '+name)
    data=json.loads(paths[name].read_text());result={}
    if data.get('inherits'):result.update(resolve_profile(data['inherits'],paths,stack+(name,)))
    for include in data.get('include',[]):result.update(resolve_profile(include,paths,stack+(name,)))
    result.update(data)
    for key in ('inherits','include'):result.pop(key,None)
    return result


def main():
    p=argparse.ArgumentParser();p.add_argument('bambu',type=Path);p.add_argument('--profiles',type=Path);p.add_argument('--report',type=Path);a=p.parse_args()
    root=Path(__file__).resolve().parents[1];cli=root/'target/release/tinkercraft-cli'
    if not cli.exists():cli=cli.with_suffix('.exe')
    bambu=str(a.bambu.resolve());fixture=root/'examples/multicolor-plate.tinkercraft'
    results={'tinkercraft':subprocess.check_output([str(cli),'--version'],text=True).strip(),
             'fixture_sha256':hashlib.sha256(fixture.read_bytes()).hexdigest(),
             'bambu_binary_sha256':hashlib.sha256(a.bambu.read_bytes()).hexdigest(),
             'imports':{},'slices':{},'interactive_filament_assignment_tested':False,'physical_print_tested':False}
    with tempfile.TemporaryDirectory(prefix='tinkercraft-slicer-') as directory:
        folder=Path(directory)
        def run(*args):
            r=subprocess.run([bambu,'--datadir',str(folder/'settings'),*map(str,args)],cwd=folder,capture_output=True,text=True,timeout=120)
            if r.returncode:raise RuntimeError(r.stdout+'\n'+r.stderr)
            return r.stdout
        help_text=run('--help');results['bambu_version']=re.search(r'BambuStudio-[\d.]+',help_text)[0]
        for kind,command,parts in [('stl','export',1),('3mf','export-3mf',2)]:
            model=folder/('plate.'+kind)
            subprocess.run([str(cli),command,str(fixture),str(model)],check=True)
            output=run('--info',model)
            fields=dict(re.findall(r'^(\w+)\s*=\s*(.+)$',output,re.M))
            for axis,expected in [('x',80),('y',30),('z',5)]:assert abs(float(fields['size_'+axis])-expected)<1e-4,output
            assert fields['manifold'].strip()=='yes' and int(fields['number_of_parts'])==parts,output
            assert abs(float(fields['volume'])-11803.790612309556)<.02,output
            results['imports'][kind]=fields
        if a.profiles:
            paths={f.stem:f for f in a.profiles.rglob('*.json')}
            for key,name in [('machine','Bambu Lab X1 Carbon 0.4 nozzle'),('process','0.20mm Standard @BBL X1C'),('filament','Generic PLA @base')]:
                data=resolve_profile(name,paths)
                (folder/(key+'.json')).write_text(json.dumps(data))
            filament=json.loads((folder/'filament.json').read_text());assert float(filament['filament_density'][0])>0
            for name,color in [('red','#DC3C32'),('blue','#1EA0D2')]:
                (folder/(name+'.json')).write_text(json.dumps({**filament,'filament_colour':[color]}))
            settings=str(folder/'machine.json')+';'+str(folder/'process.json')
            def check_slice(name,model,extra=(),expected_filaments=1):
                output=folder/name;output.mkdir()
                run(*extra,'--slice','0','--outputdir',output,model)
                data=json.loads((output/'result.json').read_text());assert data['return_code']==0,data
                plate=data['sliced_plates'][0];gcode=(output/'plate_1.gcode').read_text()
                assert '; total layer number: 25' in gcode and '; max_z_height: 5.00' in gcode
                assert '; printer_model = Bambu Lab X1 Carbon' in gcode
                used=[f for f in plate['filaments'] if f['main_used_g']>0]
                assert len(used)==expected_filaments,plate
                if expected_filaments==2:assert plate['filament_change_times']>0,plate
                assert not plate['warning_message'],plate['warning_message']
                results['slices'][name]={'layers':25,'height_mm':5,'filaments':used,'filament_changes':plate['filament_change_times'],
                                        'estimated_seconds':plate['total_predication'],'warnings':plate['warning_message']}
            profile_args=('--load-settings',settings,'--load-filaments',folder/'filament.json')
            check_slice('stl',folder/'plate.stl',profile_args)
            check_slice('3mf_single_filament',folder/'plate.3mf',profile_args)
            # Let Bambu create its own project, then assign its two existing parts
            # to slots 1 and 2. This is a programmatic assignment, not a UI test
            # and not a change to TinkerCraft's generic 3MF exporter.
            run('--load-settings',settings,'--load-filaments',str(folder/'red.json')+';'+str(folder/'blue.json'),
                '--export-3mf',folder/'bambu.3mf',folder/'plate.3mf')
            with zipfile.ZipFile(folder/'bambu.3mf') as z:contents={n:z.read(n) for n in z.namelist()}
            metadata=ET.fromstring(contents['Metadata/model_settings.config']);parts=metadata.findall('./object/part');assert len(parts)==2
            for number,part in enumerate(parts,1):
                for previous in part.findall("metadata[@key='extruder']"):part.remove(previous)
                ET.SubElement(part,'metadata',{'key':'extruder','value':str(number)})
            contents['Metadata/model_settings.config']=ET.tostring(metadata,encoding='utf-8',xml_declaration=True)
            with zipfile.ZipFile(folder/'assigned.3mf','w',zipfile.ZIP_DEFLATED) as z:
                for name,data in contents.items():z.writestr(name,data)
            check_slice('3mf_two_filaments',folder/'assigned.3mf',expected_filaments=2)
            results['slicing_profiles']={'machine':'Bambu Lab X1 Carbon 0.4 nozzle','process':'0.20mm Standard @BBL X1C','filament':'Generic PLA @base','resolved_inheritance':True}
    report=json.dumps(results,indent=2)+'\n'
    if a.report:a.report.write_text(report)
    print(report)


if __name__=='__main__':main()

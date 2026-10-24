"""Independent 3MF package/XML checks; optional official lib3mf strict-reader validation."""
import argparse, json, math, zipfile
import xml.etree.ElementTree as ET
from pathlib import Path

def validate(path, reference=False):
    ns={'m':'http://schemas.microsoft.com/3dmanufacturing/core/2015/02'}
    with zipfile.ZipFile(path) as z:
        assert z.testzip() is None
        assert {'[Content_Types].xml','_rels/.rels','3D/3dmodel.model'} <= set(z.namelist())
        ET.fromstring(z.read('[Content_Types].xml')); ET.fromstring(z.read('_rels/.rels'))
        root=ET.fromstring(z.read('3D/3dmodel.model'))
    assert root.attrib['unit']=='millimeter'
    bases=root.findall('m:resources/m:basematerials/m:base',ns)
    colors=[b.attrib['displaycolor'] for b in bases]
    assert len(colors)>=2, 'Fixture should retain multiple colors'
    objects=root.findall('m:resources/m:object',ns); ids={o.attrib['id'] for o in objects}
    volume=0.; meshes=0
    for obj in objects:
        mesh=obj.find('m:mesh',ns)
        if mesh is None:continue
        meshes+=1
        assert obj.attrib['pid']=='1' and int(obj.attrib['pindex'])<len(colors)
        vertices=[tuple(float(v.attrib[a]) for a in ['x','y','z']) for v in mesh.findall('m:vertices/m:vertex',ns)]
        assert all(math.isfinite(v) for p in vertices for v in p)
        for t in mesh.findall('m:triangles/m:triangle',ns):
            a,b,c=[vertices[int(t.attrib[k])] for k in ['v1','v2','v3']]
            volume+=(a[0]*(b[1]*c[2]-b[2]*c[1])+a[1]*(b[2]*c[0]-b[0]*c[2])+a[2]*(b[0]*c[1]-b[1]*c[0]))/6
    assert meshes>=2 and volume>0
    for c in root.findall('.//m:component',ns):assert c.attrib['objectid'] in ids
    items=root.findall('m:build/m:item',ns); assert len(items)==1 and items[0].attrib['objectid'] in ids
    if reference:
        import lib3mf
        wrapper=lib3mf.get_wrapper();model=wrapper.CreateModel();reader=model.QueryReader('3mf')
        reader.SetStrictModeActive(True);reader.ReadFromFile(str(Path(path).resolve()))
        assert reader.GetWarningCount()==0, [reader.GetWarning(i) for i in range(reader.GetWarningCount())]
        iterator=model.GetMeshObjects()
        while iterator.MoveNext():assert iterator.GetCurrentMeshObject().IsManifoldAndOriented()
    print(json.dumps(dict(file=str(path),meshes=meshes,colors=colors,volume_mm3=volume,reference_reader=reference)))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('path');p.add_argument('--reference',action='store_true');a=p.parse_args();validate(a.path,a.reference)

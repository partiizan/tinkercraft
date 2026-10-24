"""Linux GTK portal acceptance, under an isolated D-Bus session and X display.
Run: dbus-run-session -- xvfb-run -a python3 scripts/native-dialog-smoke.py
Requires xdg-desktop-portal, xdg-desktop-portal-gtk, xdotool and a release build.
Exercises actual OS dialogs, not mocked file-picker responses.
"""
import json,os,subprocess,tempfile,time,zipfile
from pathlib import Path
root=Path(__file__).resolve().parents[1]
with tempfile.TemporaryDirectory(prefix='tinkercraft-dialogs-') as directory:
    folder=Path(directory);env=os.environ.copy()
    env.update(XDG_CURRENT_DESKTOP='GNOME',LIBGL_ALWAYS_SOFTWARE='1',NO_AT_BRIDGE='1',GSETTINGS_BACKEND='memory')
    for key,name in [('XDG_CONFIG_HOME','config'),('XDG_DATA_HOME','data'),('XDG_RUNTIME_DIR','runtime')]:
        env[key]=str(folder/name);Path(env[key]).mkdir(mode=0o700)
    config=Path(env['XDG_CONFIG_HOME'])/'xdg-desktop-portal';config.mkdir()
    (config/'portals.conf').write_text('[preferred]\ndefault=gtk\n')
    log=(folder/'desktop.log').open('w');processes=[]
    def launch(args):
        p=subprocess.Popen(args,env=env,stdout=log,stderr=log);processes.append(p);return p
    def xd(*args):
        return subprocess.check_output(['xdotool',*map(str,args)],env=env,text=True,stderr=subprocess.DEVNULL).strip()
    def wait_for(check):
        end=time.monotonic()+20
        while time.monotonic()<end:
            try:
                result=check()
                if result:return result
            except (FileNotFoundError,StopIteration,subprocess.CalledProcessError,json.JSONDecodeError):pass
            time.sleep(.1)
        raise AssertionError('Timed out: '+check.__name__)
    def key(k):xd('key',k);time.sleep(.2)
    def click(x,y):xd('mousemove',x,y,'click',1);time.sleep(.2)
    def windows():
        ids=xd('search','--onlyvisible','--name','.+').splitlines()
        return [(i,xd('getwindowname',i)) for i in ids]
    def dialog():return next((i for i,name in windows() if name in ('Save project','Open project','Import STL','Export STL','Export 3MF')),None)
    def pick(path):
        print('Picker:',path.name,flush=True)
        window=wait_for(dialog);xd('windowfocus',window);key('ctrl+l');key('ctrl+a');xd('type','--clearmodifiers','--delay','1',str(path));key('Return')
        time.sleep(.4)
        if dialog():key('Return')
        wait_for(lambda:not dialog())
        xd('windowfocus',main)
    def snapshot():return json.loads(next((folder/'data').rglob('recovery.tinkercraft')).read_text())
    try:
        # D-Bus starts before xvfb-run; propagate the display and isolated data
        # paths before it auto-activates the GTK backend.
        keys=['DISPLAY','XAUTHORITY','XDG_CURRENT_DESKTOP','XDG_CONFIG_HOME','XDG_DATA_HOME','XDG_RUNTIME_DIR','NO_AT_BRIDGE','GSETTINGS_BACKEND']
        subprocess.run(['dbus-update-activation-environment',*[k for k in keys if k in env]],env=env,check=True)
        launch(['/usr/libexec/xdg-desktop-portal','--verbose'])
        time.sleep(2)
        app=launch([str(root/'target/release/tinkercraft')])
        main=wait_for(lambda:next((i for i,name in windows() if 'TinkerCraft' in name),None));xd('windowfocus',main)
        wait_for(lambda:snapshot()['parts'][0]['size'][0]==80)
        click(800,850);key('ctrl+a');key('ctrl+g')
        wait_for(lambda:len(snapshot()['parts'])==1 and snapshot()['parts'][0]['kind']=='group')
        saved=folder/'plate.tinkercraft';key('ctrl+s');pick(saved)
        wait_for(saved.exists);assert len(json.loads(saved.read_text())['parts'])==1
        key('Right');key('ctrl+s')
        wait_for(lambda:json.loads(saved.read_text())['parts'][0]['position'][0]==41)
        # Reopen externally changed content to prove the OS picker feeds Open.
        document=json.loads(saved.read_text());document['name']='Dialog reopened';saved.write_text(json.dumps(document))
        key('ctrl+o');pick(saved);wait_for(lambda:snapshot()['name']=='Dialog reopened')
        copy=folder/'copy.tinkercraft';key('ctrl+shift+s');pick(copy);wait_for(copy.exists)
        assert json.loads(copy.read_text())==document
        # Cancelling Save As must leave the existing project intact.
        key('ctrl+shift+s');wait_for(dialog);key('Escape');wait_for(lambda:not dialog());assert json.loads(copy.read_text())==document
        xd('windowfocus',main)
        stl=folder/'plate.stl';key('ctrl+shift+e');pick(stl);wait_for(stl.exists)
        three=folder/'plate.3mf';key('ctrl+alt+e');pick(three);wait_for(three.exists)
        with zipfile.ZipFile(three) as z:assert z.testzip() is None and '3D/3dmodel.model' in z.namelist()
        imported=folder/'imported.tinkercraft'
        subprocess.run([str(root/'target/release/tinkercraft-cli'),'import',str(stl),str(imported)],check=True)
        assert json.loads(imported.read_text())['parts'][0]['kind']=='mesh'
        # Exercise the native import picker; review/add remains covered by native-smoke.
        key('ctrl+i');pick(stl);key('Escape')
        print('PASS: GTK portal Save/Open/Save As/cancel, grouped edit persisted, STL/3MF export and STL import picker')
    except Exception:
        log.flush();print((folder/'desktop.log').read_text());print(windows())
        raise
    finally:
        for p in reversed(processes):
            if p.poll() is None:p.terminate();p.wait(timeout=10)
        mount=folder/'runtime/doc'
        if os.path.ismount(mount):subprocess.run(['fusermount3','-u',str(mount)],check=True)
        log.close()

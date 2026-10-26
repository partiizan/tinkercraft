"""Linux workplane, scoped editing, outline import and renderer regression at 1440x960.
Requires Xvfb and xdotool in PATH; no Python packages. Uses isolated app data.
Build first: cargo build --release --workspace --locked
Run: python3 scripts/native-smoke.py
TC_SMOKE_TCP=1 supports containers that isolate Unix display sockets.
TC_SMOKE_DISPLAY selects a free display number (default 119).
"""
import json, os, subprocess, tempfile, time
from pathlib import Path
root=Path(__file__).resolve().parents[1]
bin_dir=root/'target/release'
with tempfile.TemporaryDirectory(prefix='tinkercraft-smoke-') as directory:
    directory=Path(directory)
    env=os.environ.copy()
    number=env.get('TC_SMOKE_DISPLAY','119')
    tcp=env.get('TC_SMOKE_TCP')=='1'
    env.update(DISPLAY=('127.0.0.1:' if tcp else ':')+number,
               XDG_CONFIG_HOME=str(directory/'config'),XDG_RUNTIME_DIR=str(directory/'runtime'),
               LIBGL_ALWAYS_SOFTWARE='1',GDK_SCALE='1',QT_SCALE_FACTOR='1')
    for key in ['XDG_CONFIG_HOME','XDG_RUNTIME_DIR']:Path(env[key]).mkdir(mode=0o700)
    log=(directory/'native.log').open('w')
    display=subprocess.Popen(['Xvfb',':'+number,'-screen','0','1440x960x24',
                              *(['-listen','tcp','-nolisten','unix'] if tcp else ['-nolisten','tcp']),'-ac'],env=env,stdout=log,stderr=log)
    app=None
    def xd(*args):
        return subprocess.check_output(['xdotool',*map(str,args)],env=env,text=True,stderr=subprocess.DEVNULL).strip()
    def click(x,y):xd('mousemove',x,y,'click',1);time.sleep(.2)
    def key(k):xd('key',k);time.sleep(.1)
    def snapshot():
        return json.loads(next(Path(env['XDG_DATA_HOME']).rglob('recovery.tinkercraft')).read_text())
    def wait_for(check):
        deadline=time.monotonic()+15
        while time.monotonic()<deadline:
            try:
                if check():return
            except (AssertionError,FileNotFoundError,StopIteration,KeyError,IndexError,json.JSONDecodeError):pass
            time.sleep(.1)
        raise AssertionError('Timed out waiting for expected document state')
    def launch(data,*args):
        global app
        env['XDG_DATA_HOME']=str(directory/data);Path(env['XDG_DATA_HOME']).mkdir(exist_ok=True)
        env['XDG_CONFIG_HOME']=str(directory/(data+'-config'));Path(env['XDG_CONFIG_HOME']).mkdir(exist_ok=True)
        app=subprocess.Popen([str(bin_dir/'tinkercraft'),*map(str,args)],env=env,stdout=log,stderr=log)
        for _ in range(60):
            if app.poll() is not None:raise AssertionError('Native app exited early')
            try:
                window=xd('search','--onlyvisible','--name','TinkerCraft').splitlines()[0]
                xd('windowfocus',window);time.sleep(.5);return
            except (subprocess.CalledProcessError,IndexError):time.sleep(.2)
        raise AssertionError('No native window')
    def stop():
        if app and app.poll() is None:app.terminate();app.wait(timeout=10)
    def drag(x,y,xx,yy):
        xd('mousemove',x,y,'mousedown',1);time.sleep(.15)
        xd('mousemove',xx,yy);time.sleep(.25);xd('mouseup',1)
    try:
        time.sleep(1);launch('studio')
        wait_for(lambda:len(snapshot()['parts'])==3)
        key('ctrl+a');key('ctrl+g');wait_for(lambda:len(snapshot()['parts'])==1)
        key('F3');time.sleep(.5);click(270,57);click(440,57);key('w')
        drag(760,517,810,517)
        wait_for(lambda:snapshot()['parts'][0]['children'][0]['position'][0]>0)
        changed=snapshot()['parts'][0]['children'][0]['position'][0]
        key('shift+F3');time.sleep(.5)
        key('ctrl+z');wait_for(lambda:snapshot()['parts'][0]['children'][0]['position'][0]==0)
        key('ctrl+shift+z');wait_for(lambda:snapshot()['parts'][0]['children'][0]['position'][0]==changed)
        key('F3');key('Right');wait_for(lambda:snapshot()['parts'][0]['children'][0]['position'][0]>changed)
        click(745,158);wait_for(lambda:snapshot()['parts'][0]['children'][0]['position'][0]==changed)
        key('F4');time.sleep(.5)
        xd('mousemove',410,265,'click','--repeat','2','--delay','100',1);key('ctrl+a');xd('type','10');key('Return');click(285,327)
        wait_for(lambda:snapshot()['workplane']['origin'][2]==10)
        click(717,147);click(51,164)
        wait_for(lambda:len(snapshot()['parts'])==2 and snapshot()['parts'][-1]['position'][2]==20)
        key('ctrl+z');wait_for(lambda:len(snapshot()['parts'])==1)
        key('ctrl+z');wait_for(lambda:snapshot()['workplane']['origin'][2]==0)
        print('PASS scoped handle drag, full-project recovery, apply/undo/redo/cancel and custom-plane placement')
        stop();launch('svg-outline',root/'examples/outline-badge.svg')
        time.sleep(.5);xd('mousemove',600,540,'click','--repeat','6','--delay','60',5);time.sleep(.4)
        click(390,371);time.sleep(1);click(310,512)
        wait_for(lambda:len(snapshot()['parts'])==4 and snapshot()['parts'][-1]['kind']=='mesh')
        click(717,147);key('ctrl+z');wait_for(lambda:len(snapshot()['parts'])==3)
        key('ctrl+shift+z');wait_for(lambda:len(snapshot()['parts'])==4)
        key('F4');time.sleep(.4);click(254,540)
        wait_for(lambda:json.loads(next(Path(env['XDG_CONFIG_HOME']).rglob('preferences.json')).read_text())['gpu_viewport'] is False)
        print('PASS native SVG generation/add/undo/redo and software fallback preference')
    except Exception:
        log.flush();print((directory/'native.log').read_text());raise
    finally:
        stop();display.terminate();display.wait(timeout=10);log.close()

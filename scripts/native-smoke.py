"""Linux native-window regression test at 1440x960/default scale.
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
        time.sleep(1)
        launch('edit')
        wait_for(lambda:snapshot()['parts'][0]['size'][0]==80)
        xd('mousemove',1270,123,'click','--repeat','2','--delay','100',1)
        key('ctrl+a');xd('type','--clearmodifiers','100');key('Return')
        wait_for(lambda:snapshot()['parts'][0]['size'][0]==100)
        click(27,99);wait_for(lambda:snapshot()['parts'][0]['size'][0]==80)
        click(72,99);wait_for(lambda:snapshot()['parts'][0]['size'][0]==100)
        click(270,57);click(440,57);key('r')
        drag(760,517,810,517)
        wait_for(lambda:snapshot()['parts'][0]['size'][0]>100)
        resized=snapshot()['parts'][0]['size'][0]
        click(27,99);wait_for(lambda:snapshot()['parts'][0]['size'][0]==100)
        click(72,99);wait_for(lambda:snapshot()['parts'][0]['size'][0]==resized)
        click(27,99);wait_for(lambda:snapshot()['parts'][0]['size'][0]==100)
        xd('mousemove',760,517,'mousedown',1);time.sleep(.1);xd('mousemove',810,517);time.sleep(.2)
        key('Escape');xd('mouseup',1);time.sleep(1)
        assert snapshot()['parts'][0]['size'][0]==100
        click(24,430);wait_for(lambda:snapshot()['parts'][0]['hidden'])
        click(24,430);wait_for(lambda:not snapshot()['parts'][0]['hidden'])
        click(66,430);wait_for(lambda:snapshot()['parts'][0]['locked'])
        key('Right');key('Delete');time.sleep(.8)
        assert len(snapshot()['parts'])==3 and snapshot()['parts'][0]['position'][0]==40
        click(68,430);wait_for(lambda:not snapshot()['parts'][0]['locked'])
        key('Right');wait_for(lambda:snapshot()['parts'][0]['position'][0]==41)
        click(27,99);wait_for(lambda:snapshot()['parts'][0]['position'][0]==40)
        def prefs():return json.loads(next(Path(env['XDG_CONFIG_HOME']).rglob('preferences.json')).read_text())
        wait_for(lambda:prefs()['tool']==2 and prefs()['orthographic'])
        click(51,260);wait_for(lambda:snapshot()['parts'][-1]['kind']=='wedge')
        click(150,260);wait_for(lambda:snapshot()['parts'][-1]['kind']=='roof')
        click(37,326);click(29,346)
        wait_for(lambda:len(snapshot()['parts'])==1 and snapshot()['parts'][0]['kind']=='group')
        click(86,346);wait_for(lambda:len(snapshot()['parts'])==5)
        stop()
        launch('group-edit')
        key('ctrl+a');key('ctrl+g')
        wait_for(lambda:len(snapshot()['parts'])==1 and snapshot()['parts'][0]['kind']=='group')
        click(270,57);click(440,57);key('w')
        original_x=snapshot()['parts'][0]['position'][0]
        drag(760,517,810,517)
        wait_for(lambda:snapshot()['parts'][0]['position'][0]>original_x)
        key('ctrl+z');wait_for(lambda:snapshot()['parts'][0]['position'][0]==original_x)
        key('r');drag(760,517,810,517)
        wait_for(lambda:snapshot()['parts'][0]['scale']>1)
        key('ctrl+z');wait_for(lambda:snapshot()['parts'][0]['scale']==1)
        key('ctrl+shift+g');wait_for(lambda:len(snapshot()['parts'])==3)
        stop()
        sample=directory/'sample.stl'
        subprocess.run([str(bin_dir/'tinkercraft-cli'),'export',str(root/'examples/plate.tinkercraft'),str(sample)],check=True)
        launch('import',sample)
        click(590,518);time.sleep(1);click(593,508)
        wait_for(lambda:len(snapshot()['parts'])==4 and snapshot()['parts'][-1]['kind']=='mesh')
        click(27,99);wait_for(lambda:len(snapshot()['parts'])==3)
        click(72,99);wait_for(lambda:len(snapshot()['parts'])==4)
        click(30,429);click(270,57);click(440,57);key('r')
        drag(760,517,810,517)
        wait_for(lambda:snapshot()['parts'][-1]['size'][0]>80)
        click(27,99);wait_for(lambda:snapshot()['parts'][-1]['size'][0]==80)
        stop();launch('import')
        click(595,501);time.sleep(.5) # Restore recovery, then edit it to prove restoration.
        # A remembered Resize tool must work without pressing R after restart.
        click(30,429)
        drag(760,517,810,517)
        wait_for(lambda:snapshot()['parts'][-1]['size'][0]>80)
        click(27,99);wait_for(lambda:snapshot()['parts'][-1]['size'][0]==80)
        click(51,164)
        wait_for(lambda:len(snapshot()['parts'])==5 and snapshot()['parts'][3]['kind']=='mesh')
        stop();launch('appearance-color')
        wait_for(lambda:len(snapshot()['parts'])==3)
        original_color=snapshot()['parts'][0]['color']
        click(275,18);click(620,434);click(545,551)
        wait_for(lambda:json.loads(next(Path(env['XDG_CONFIG_HOME']).rglob('preferences.json')).read_text())['appearance']=='light')
        click(1262,375);click(1020,720);click(800,850)
        wait_for(lambda:snapshot()['parts'][0]['color']!=original_color)
        changed_color=snapshot()['parts'][0]['color']
        key('ctrl+z');wait_for(lambda:snapshot()['parts'][0]['color']==original_color)
        key('ctrl+shift+z');wait_for(lambda:snapshot()['parts'][0]['color']==changed_color)
        stop();launch('appearance-color');click(595,501)
        wait_for(lambda:snapshot()['parts'][0]['color']==changed_color)
        assert json.loads(next(Path(env['XDG_CONFIG_HOME']).rglob('preferences.json')).read_text())['appearance']=='light'
        stop();launch('group-contents')
        wait_for(lambda:len(snapshot()['parts'])==3)
        key('ctrl+a');key('ctrl+g');wait_for(lambda:len(snapshot()['parts'])==1)
        key('F2');time.sleep(1)
        xd('mousemove',616,592,'click','--repeat','2','--delay','100',1)
        key('ctrl+a');xd('type','10');key('Return');click(440,750)
        wait_for(lambda:snapshot()['parts'][0]['children'][1]['size'][0]==10)
        key('ctrl+z');wait_for(lambda:snapshot()['parts'][0]['children'][1]['size'][0]==5)
        key('ctrl+shift+z');wait_for(lambda:snapshot()['parts'][0]['children'][1]['size'][0]==10)
        print('PASS: nested hole dimension edit with group-preserving undo/redo')
        print('PASS: numeric undo/redo, resize undo/redo/cancel, wedge/roof palette, group/ungroup, STL review/add/undo, imported mesh resizing, recovery restore, hide/show, lock enforcement, nudge, persisted camera/tool, appearance and color undo/redo/recovery')
    except Exception:
        try:
            from PIL import ImageGrab
            ImageGrab.grab(xdisplay=env['DISPLAY']).save('/tmp/tinkercraft-smoke-failure.png')
            print([(n['name'],n['kind'],n.get('size')) for n in snapshot()['parts']])
        except Exception:pass
        log.flush();print((directory/'native.log').read_text())
        raise
    finally:
        stop();display.terminate();display.wait(timeout=10);log.close()

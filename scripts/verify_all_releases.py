"""Verify and execute packages produced by the current native CI job."""
from pathlib import Path
import subprocess,sys
root=Path(__file__).resolve().parents[1]
archives=sorted([*root.glob('dist/*.zip'),*root.glob('dist/*.tar.gz')])
if not archives:raise SystemExit('No release packages found')
for path in archives:subprocess.run([sys.executable,str(root/'scripts/verify_release.py'),str(path),'--run'],check=True)

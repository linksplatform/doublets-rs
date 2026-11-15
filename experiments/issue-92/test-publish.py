"""Exercise the publisher with a mock cargo; never contact a registry."""
import os
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix='doublets-publish-') as temp:
    temp = Path(temp)
    publisher = temp / 'publisher'
    subprocess.run(['rustc', str(root / 'scripts/publish-crate.rs'), '-o', str(publisher)], check=True)
    cargo = temp / 'cargo'
    cargo.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$PUBLISH_CALLS"\nexit "${MOCK_EXIT:-0}"\n')
    cargo.chmod(0o755)
    env = dict(os.environ, PATH=str(temp) + os.pathsep + os.environ['PATH'],
               PUBLISH_CALLS=str(temp / 'calls'), GITHUB_OUTPUT=str(temp / 'output'))
    env.pop('CARGO_REGISTRY_TOKEN', None)
    result = subprocess.run([str(publisher)], env=env, capture_output=True, text=True)
    assert result.returncode == 1 and 'trusted publisher' in result.stderr
    assert not (temp / 'calls').exists()
    env['CARGO_REGISTRY_TOKEN'] = 'mock-short-lived-token'
    result = subprocess.run([str(publisher)], env=env, capture_output=True, text=True)
    assert result.returncode == 0, result.stderr
    assert (temp / 'calls').read_text().splitlines() == [
        'publish', '--package', 'doublets', '--locked', '--allow-dirty']
    assert env['CARGO_REGISTRY_TOKEN'] not in result.stdout + result.stderr
    assert 'publish_result=success' in (temp / 'output').read_text()
    env['MOCK_EXIT'] = '42'
    result = subprocess.run([str(publisher)], env=env, capture_output=True, text=True)
    assert result.returncode == 42
    assert (temp / 'output').read_text().endswith('publish_result=failed\n')
print('Publisher requires authentication, selects only doublets, and propagates cargo failures.')

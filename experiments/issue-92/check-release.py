"""Regression checks for the single-package release and self-contained FFI source."""
import json
from pathlib import Path
import subprocess
import sys
import tomllib

root = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path(__file__).resolve().parents[2]
metadata = json.loads(subprocess.check_output(
    ['cargo', 'metadata', '--no-deps', '--format-version', '1'], cwd=root))
assert [p['name'] for p in metadata['packages'] if p['publish'] != []] == ['doublets']
for package in ['doublets-ffi', 'doublets-ffi/ffi-attributes', 'doublets-ffi/env-decorators']:
    manifest = tomllib.loads((root / package / 'Cargo.toml').read_text())
    assert manifest['package']['publish'] is False, package
primary = next(p for p in metadata['packages'] if p['name'] == 'doublets')
assert 'ffi' in primary['features']
assert all(d['path'] is None for d in primary['dependencies'] if 'path' in d)
assert (root / 'doublets/src/ffi/mod.rs').is_file()
workflow = (root / '.github/workflows/release.yml').read_text()
assert workflow.count('uses: rust-lang/crates-io-auth-action@v1') == 2
assert workflow.count('id-token: write') == 2
assert workflow.count('CARGO_REGISTRY_TOKEN: ${{ steps.crates-auth.outputs.token }}') == 2
assert 'secrets.CARGO_TOKEN' not in workflow
assert 'secrets.CARGO_REGISTRY_TOKEN' not in workflow
print('Only doublets is publishable; FFI has no path dependencies; both release paths use OIDC.')

"""Query crates.io for stable releases of every declared Rust dependency.

Run from the repository root; keep the JSON snapshot for a reproducible audit.
Includes excluded crates and embedded rust-script manifests.
"""
import concurrent.futures
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import re
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[2]


def declarations():
    manifests = list(ROOT.rglob('Cargo.toml'))
    manifests = [p for p in manifests if 'target' not in p.relative_to(ROOT).parts]
    for path in sorted(manifests):
        data = tomllib.loads(path.read_text())
        for section in ('dependencies', 'dev-dependencies', 'build-dependencies'):
            for name, spec in data.get(section, {}).items():
                if isinstance(spec, str):
                    spec = {'version': spec}
                if 'version' in spec:
                    yield str(path.relative_to(ROOT)), section, name, spec
    for path in sorted((ROOT / 'scripts').glob('*.rs')):
        match = re.search(r'//! ```cargo\n(.*?)//! ```', path.read_text(), re.S)
        if not match:
            continue
        data = tomllib.loads(re.sub(r'^//! ?', '', match[1], flags=re.M))
        for name, spec in data.get('dependencies', {}).items():
            if isinstance(spec, str):
                spec = {'version': spec}
            yield str(path.relative_to(ROOT)), 'script', name, spec


def fetch(name):
    request = urllib.request.Request(
        f'https://crates.io/api/v1/crates/{name}',
        headers={'User-Agent': 'doublets-rs-issue-68-dependency-audit'},
    )
    with urllib.request.urlopen(request) as response:
        data = json.load(response)
    stable = [v for v in data['versions'] if not v['yanked'] and '-' not in v['num']]
    latest = max(stable, key=lambda v: tuple(map(int, v['num'].split('.'))))
    return name, {'version': latest['num'], 'rust_version': latest['rust_version'],
                  'source': f'https://crates.io/crates/{name}/{latest["num"]}'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true',
                        help='check declarations against the saved release snapshot without network access')
    args = parser.parse_args()
    declared = list(declarations())
    output = ROOT / 'experiments/issue-68/dependencies.json'
    if args.check:
        releases = json.loads(output.read_text())['releases']
        for file, _, name, spec in declared:
            latest = releases[spec.get('package', name)]['version']
            if spec['version'] != latest:
                raise SystemExit(f'{file}: {name} requires {spec["version"]}, expected {latest}')
        print(f'All {len(declared)} dependency declarations match the stable release snapshot')
        raise SystemExit(0)
    names = sorted({spec.get('package', name) for _, _, name, spec in declared})
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        releases = dict(pool.map(fetch, names))
    snapshot = {'audited_at': datetime.now(timezone.utc).isoformat(),
                'releases': dict(sorted(releases.items())), 'declarations': [
        {'file': file, 'section': section, 'alias': name,
         'crate': spec.get('package', name), 'requirement': spec['version'],
         'latest_stable': releases[spec.get('package', name)]['version']}
        for file, section, name, spec in declared]}
    output.write_text(json.dumps(snapshot, indent=2) + '\n')
    for name, info in sorted(releases.items()):
        print(f'{name}: {info["version"]} (Rust {info["rust_version"]})')

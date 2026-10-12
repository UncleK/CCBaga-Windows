#!/usr/bin/env python3
"""Pinned Windows engine: SHA-256 before extraction, then PE and resource checks."""
import argparse
import hashlib
import json
import shutil
import stat
import struct
import tempfile
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parent.parent
VERSION = '148.0.7778.215'
URL = f'https://github.com/adryfish/fingerprint-chromium/releases/download/{VERSION}/ungoogled-chromium_{VERSION}-1.1_windows_x64.zip'
SHA256 = '9ef3f471b7a6641b4224532522b29141ce3746e27d55788d88e2fd951f362579'

def extract(archive, destination):
    with zipfile.ZipFile(archive) as bundle:
        for entry in bundle.infolist():
            path = PurePosixPath(entry.filename.replace('\\', '/'))
            if path.is_absolute() or '..' in path.parts or any(':' in p for p in path.parts):
                raise ValueError('Unsafe ZIP path')
            if stat.S_ISLNK(entry.external_attr >> 16):
                raise ValueError('Browser ZIP must not contain links')
            for part in path.parts:
                if part.endswith((' ', '.')) or part.split('.')[0].upper() in {'CON','PRN','AUX','NUL',*[f'COM{i}' for i in range(1,10)],*[f'LPT{i}' for i in range(1,10)]}:
                    raise ValueError('Unsafe Windows ZIP name')
        bundle.extractall(destination)

def verify_pe(path):
    with path.open('rb') as stream:
        if stream.read(2) != b'MZ': raise ValueError('Expected Windows PE')
        stream.seek(0x3c)
        offset = struct.unpack('<I', stream.read(4))[0]
        stream.seek(offset)
        if stream.read(6) != b'PE\x00\x00\x64\x86': raise ValueError('Expected Windows x64 PE')

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--archive', type=Path, help='Offline ZIP, still SHA-256 verified')
    args = parser.parse_args()
    target = ROOT / 'browser-runtime/windows'
    receipt = {'version':VERSION, 'sha256':SHA256}
    if target.exists():
        marker = target / 'ccbaga-engine.json'
        if marker.is_file() and json.loads(marker.read_text()) == receipt:
            verify_pe(target / 'chrome.exe')
            if not (target/'icudtl.dat').is_file() or not (target/'locales').is_dir():
                raise SystemExit('Prepared engine resources are missing')
            print(f'Prepared Windows engine {VERSION} already exists')
            return
        raise SystemExit('Existing engine is not a verified prepared download; move it aside first')
    target.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='ccbaga-engine-',dir=target.parent) as temporary:
        temp = Path(temporary)
        archive = args.archive or temp/'browser.zip'
        if not args.archive:
            print(f'Downloading Windows x64 engine {VERSION}...',flush=True)
            with urllib.request.urlopen(URL,timeout=60) as response, archive.open('wb') as output:
                shutil.copyfileobj(response,output)
        with archive.open('rb') as stream:
            if hashlib.file_digest(stream,'sha256').hexdigest() != SHA256:
                raise SystemExit('SHA-256 mismatch; refusing extraction')
        unpacked = temp/'unpacked'
        extract(archive,unpacked)
        candidates = list(unpacked.rglob('chrome.exe'))
        if len(candidates) != 1: raise SystemExit('Expected one chrome.exe')
        verify_pe(candidates[0])
        engine = candidates[0].parent
        if not (engine/'icudtl.dat').is_file() or not (engine/'locales').is_dir():
            raise SystemExit('Incomplete engine resources')
        (engine/'ccbaga-engine.json').write_text(json.dumps(receipt),encoding='utf-8')
        shutil.move(str(engine),str(target))
    print(f'Windows engine {VERSION}: SHA-256, PE architecture and resources verified')

if __name__ == '__main__': main()

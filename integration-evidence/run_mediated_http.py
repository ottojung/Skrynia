#!/usr/bin/env python3
"""Real HTTP P0 -> P1 -> P2 mediated-callback acceptance + malicious-P2 negative.

Isolated, disposable, no mocks. All artifacts pinned by SHA256. Uses the
proven PR118 DATA_DIR layout (continuation10 harness pattern). Kernel is
rebuilt from an isolated checkout at 249ba34 (contains reentrancy fix 7695fd3).

Positive: register/lookup/invoke P1->P2 echo/appdata CAS/restart durability.
Negative: compiled adversarial P2 (callback_abuse) invokes P1's mediator
callback (which re-enters P1 -> P0 for storage) and tries cross-app reads,
writes, deletes, non-appdata ops, private paths and raw FD guessing.
"""
import hashlib, pathlib, socket, struct, os, shutil, subprocess, time, sys

WD = pathlib.Path(os.environ.get('MEDIATED_WD', '/tmp/opencode/p1p2-http-8TasxocIctWh2VHtJ7e4EX-znh5RvkLAxPWER7dQJAcsq-20261121'))
KIT = pathlib.Path('/workspace/skrynia-v3-integration-kit')
KERNEL = pathlib.Path(os.environ.get('MEDIATED_KERNEL', str(WD / 'target-kernel/release/skrynia-server')))
P0 = pathlib.Path(os.environ.get('MEDIATED_P0', str(KIT / 'artifacts/skrynia_program0_pr51_f4a5270_wire_reconcile_aa6333a_missdir_trace_clamp_v2.wasm')))
P1 = pathlib.Path(os.environ.get('MEDIATED_P1', '/tmp/opencode/p1-export-table-3O5p3mpYtZuUeg97CGnZyc-20261121/publish/skrynia_native_service_p1_aa6333a_export_table.wasm'))
ECHO = pathlib.Path(os.environ.get('MEDIATED_ECHO', str(KIT / 'tests/guests/echo_guest.wasm')))
ATTACK = pathlib.Path(os.environ.get('MEDIATED_ATTACK', str(KIT / 'wasm-target/wasm32-unknown-unknown/release/callback_abuse.wasm')))

SHA = {
    'kernel': ('91cd9b8a179a1db5dfcc2c2630746a3744cfc8e785d2c476542d0c1703e1e3e4', KERNEL),
    'p0': ('6e4f41e280df56ec7feb5a91e618add2b1d24c934561b4c56767de5c40a936f7', P0),
    'p1': ('6b5f793b64a044fb0ca4cb483c0af0cedfa4c3a846ac102a16eb74884a67c9aa', P1),
    'echo': ('dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5', ECHO),
    'attack': ('f9eaa6438c96ef000085e30b61ca49a43a174786a10ac9a19dae128f92a3d52b', ATTACK),
}

def put_id(b): return struct.pack('<H', len(b)) + b
def put_blob(b): return struct.pack('<I', len(b)) + b
def sha256(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()

def check_hashes():
    for name, (want, path) in SHA.items():
        if not path.is_file():
            raise SystemExit(f'MISSING {name}: {path}')
        got = sha256(path)
        print(f'ARTIFACT {name} {got} {"OK" if got == want else "MISMATCH want " + want}', flush=True)
        if got != want:
            raise SystemExit(f'SHA mismatch {name}: want {want} got {got}')

def dec(r):
    if len(r) < 6: return {'kind': 'short', 'hex': r.hex()}
    status = r[1]; blen = struct.unpack('<I', r[2:6])[0]; body = r[6:6 + blen]
    if status == 0: return {'kind': 'ok', 'blen': blen, 'body': body, 'body_hex': body.hex()}
    if status == 1 and body: return {'kind': 'err', 'code': body[0], 'extra': body[1:].hex()}
    return {'kind': 'other', 'status': status, 'hex': body.hex()}

class Srv:
    def __init__(self, base, port, token, proc=None):
        self.base, self.port, self.token, self.proc = base, port, token, proc
    def stop(self):
        if self.proc is None: return
        if self.proc.poll() is None:
            self.proc.terminate()
            try: self.proc.wait(timeout=5)
            except Exception: self.proc.kill(); self.proc.wait(timeout=5)
    def send(self, payload, label='', timeout=30):
        body = b'\x01' + self.token + payload
        s = socket.create_connection(('127.0.0.1', self.port), 5); s.settimeout(timeout)
        s.sendall(f'POST / HTTP/1.1\r\nHost: 127.0.0.1:{self.port}\r\nContent-Length: {len(body)}\r\nConnection: close\r\n\r\n'.encode() + body)
        resp = b''
        while True:
            b = s.recv(65536)
            if not b: break
            resp += b
        s.close()
        h, r = resp.split(b'\r\n\r\n', 1)
        if b'transfer-encoding: chunked' in h.lower():
            out = bytearray(); i = 0
            while True:
                j = r.find(b'\r\n', i)
                if j < 0: break
                sz = int(r[i:j].split(b';', 1)[0], 16)
                if sz == 0: break
                out.extend(r[j + 2:j + 2 + sz]); i = j + 2 + sz + 2
            r = bytes(out)
        d = dec(r); sl = h.split(b'\r\n', 1)[0].decode()
        if label: print(f'  {label}: {sl} {d}', flush=True)
        return d

def free_port():
    s = socket.socket(); s.bind(('127.0.0.1', 0)); p = s.getsockname()[1]; s.close(); return p

def setup(name, port, operator_cap=None):
    base = WD / ('http-' + name)
    if base.exists(): shutil.rmtree(base)
    base.mkdir(parents=True)
    data = base / 'data'
    (data / 'security' / 'program').mkdir(parents=True)
    (data / 'programs').mkdir(parents=True)
    reg = data / 'userland' / 'programs'
    for s in ['active', 'blob', 'ready', 'appdata']: (reg / s).mkdir(parents=True, exist_ok=True)
    if operator_cap: (reg / 'operator.cap').write_bytes(operator_cap)
    token = hashlib.sha256(name.encode()).digest()
    (data / 'security' / 'program0.sha256').write_bytes(hashlib.sha256(token).digest())
    (data / 'programs' / 'root.wasm').write_bytes(P1.read_bytes())
    meta = struct.pack('<II', 0xfff, len(b'userland')) + b'userland'
    (data / 'security' / 'program' / 'programs%2Froot.wasm').write_bytes(meta)
    return base, data, token

def start(base, data, port):
    env = {**os.environ, 'SKRYNIA_ROOT_WASM': str(P0), 'SKRYNIA_DATA_DIR': str(data), 'SKRYNIA_BIND': f'127.0.0.1:{port}'}
    so = open(base / 'stdout', 'wb'); se = open(base / 'stderr', 'wb')
    proc = subprocess.Popen([str(KERNEL)], env=env, stdout=so, stderr=se)
    for _ in range(150):
        if proc.poll() is not None:
            raise RuntimeError(f'server died rc={proc.returncode} stderr={pathlib.Path(base/"stderr").read_text(errors="replace")[-800:]}')
        try: socket.create_connection(('127.0.0.1', port), 0.2).close(); return proc
        except Exception: time.sleep(0.2)
    raise RuntimeError('server did not listen')

def rec(results, name, ok, detail=''):
    results.append((name, bool(ok), str(detail)))
    print(('PASS' if ok else 'FAIL'), name, detail, flush=True)

# ---------------- Positive real HTTP flow ----------------
def positive(results):
    print('\n=== POSITIVE: real HTTP P0 -> P1 -> P2 mediated callback ===', flush=True)
    port = free_port()
    op_cap = hashlib.sha256(b'op-pos').digest()
    admin = hashlib.sha256(b'admin-pos').digest(); iid = hashlib.sha256(admin).digest()
    base, data, token = setup('pos', port, operator_cap=op_cap)
    proc = start(base, data, port)
    echo = ECHO.read_bytes()
    try:
        srv = Srv(base, port, token)
        d = srv.send(b'\x01\x0f' + put_id(op_cap) + put_id(admin) + put_blob(echo), 'register')
        rec(results, 'positive-register', d.get('kind') == 'ok', d)
        d = srv.send(b'\x01\x04' + put_id(iid), 'lookup')
        rec(results, 'positive-lookup', d.get('kind') == 'ok', d)
        d = srv.send(b'\x01\x03' + put_id(iid) + put_blob(b'hello-p1-p2'), 'invoke-p1-p2')
        rec(results, 'positive-invoke-p1-p2-echo', d.get('kind') == 'ok' and d.get('body') == b'hello-p1-p2', d)
        # appdata put (absent), get, CAS with correct/wrong precondition
        srv.send(b'\x01\x06' + put_id(admin) + put_blob(b'k') + b'\x00' + put_blob(b'v1'), 'put1')
        d = srv.send(b'\x01\x07' + put_id(iid) + put_blob(b'k'), 'get1')
        rec(results, 'positive-appdata-put-get', d.get('kind') == 'ok' and d.get('body') == b'v1', d)
        h = hashlib.sha256(b'v1').digest()
        srv.send(b'\x01\x06' + put_id(admin) + put_blob(b'k') + b'\x01' + h + put_blob(b'v2'), 'cas-ok')
        d = srv.send(b'\x01\x07' + put_id(iid) + put_blob(b'k'), 'get2')
        rec(results, 'positive-appdata-cas', d.get('kind') == 'ok' and d.get('body') == b'v2', d)
        srv.send(b'\x01\x06' + put_id(admin) + put_blob(b'k') + b'\x01' + b'\x11' * 32 + put_blob(b'v3'), 'cas-wrong')
        d = srv.send(b'\x01\x07' + put_id(iid) + put_blob(b'k'), 'get3')
        rec(results, 'positive-cas-wrong-precond-rejected', d.get('kind') == 'ok' and d.get('body') == b'v2', d)
        srv.stop()
        # durable restart
        proc = start(base, data, port)
        srv = Srv(base, port, token)
        d = srv.send(b'\x01\x04' + put_id(iid), 'lookup-restart')
        rec(results, 'positive-lookup-after-restart', d.get('kind') == 'ok', d)
        d = srv.send(b'\x01\x03' + put_id(iid) + put_blob(b'after-restart'), 'invoke-restart')
        rec(results, 'positive-invoke-after-restart', d.get('kind') == 'ok' and d.get('body') == b'after-restart', d)
        d = srv.send(b'\x01\x07' + put_id(iid) + put_blob(b'k'), 'get-restart')
        rec(results, 'positive-appdata-after-restart', d.get('kind') == 'ok' and d.get('body') == b'v2', d)
    finally:
        if proc.poll() is None: proc.stop() if hasattr(proc, 'stop') else None
        if proc.poll() is None: proc.terminate(); proc.wait(timeout=5)

# ---------------- Malicious P2 reentrant cross-app negative ----------------
def negative(results):
    print('\n=== NEGATIVE: malicious P2 reentrant cross-app / sibling denial ===', flush=True)
    port = free_port()
    op_cap = hashlib.sha256(b'op-adv').digest()
    adminA = hashlib.sha256(b'admin-A-victim').digest(); iidA = hashlib.sha256(adminA).digest()
    adminB = hashlib.sha256(b'admin-B-attacker').digest(); iidB = hashlib.sha256(adminB).digest()
    readB = hashlib.sha256(adminB).digest(); writeB = adminB
    readA = hashlib.sha256(adminA).digest(); writeA = adminA
    base, data, token = setup('adv', port, operator_cap=op_cap)
    proc = start(base, data, port)
    echo = ECHO.read_bytes(); attack = ATTACK.read_bytes()
    try:
        srv = Srv(base, port, token)
        d = srv.send(b'\x01\x0f' + put_id(op_cap) + put_id(adminA) + put_blob(echo), 'reg-victim-A')
        rec(results, 'neg-register-victim-A', d.get('kind') == 'ok', d)
        d = srv.send(b'\x01\x0f' + put_id(op_cap) + put_id(adminB) + put_blob(attack), 'reg-attacker-B')
        rec(results, 'neg-register-attacker-P2', d.get('kind') == 'ok', d)
        d = srv.send(b'\x01\x03' + put_id(iidA) + put_blob(b'ping'), 'victim-A-sanity')
        rec(results, 'neg-victim-A-sanity-invoke', d.get('kind') == 'ok' and d.get('body') == b'ping', d)
        # Seed victim A appdata key 'secret' so cross-app read has something to steal
        srv.send(b'\x01\x06' + put_id(adminA) + put_blob(b'secret') + b'\x00' + put_blob(b'TOPSECRET-A'), 'seed-A-secret')
        # adversarial input: own/iid, sib, own_r, own_w, sib_r, sib_w, op
        inp = iidB + iidA + readB + writeB + readA + writeA + op_cap
        d = srv.send(b'\x01\x03' + put_id(iidB) + put_blob(inp), 'invoke-attacker-B', timeout=60)
        rec(results, 'neg-attacker-P2-invoke-via-P1-callback', d.get('kind') == 'ok', d)
        report = {}
        if d.get('kind') == 'ok':
            txt = d['body'].decode('utf-8', 'replace')
            print('  --- attacker report ---', flush=True)
            for line in txt.splitlines(): print('  ' + line, flush=True)
            print('  --- end report ---', flush=True)
            for line in txt.splitlines():
                if '=' in line:
                    k, v = line.split('=', 1)
                    try: report[k.strip()] = int(v.strip())
                    except ValueError: pass
        # Reentrant P1->P0 storage path works for self (own iid as cap) : absent -> err 8, self put -> 0
        rec(results, 'neg-reentrant-self-read-returns-absent-not-fault', report.get('get_own_iid_as_readcap') == 8, report.get('get_own_iid_as_readcap'))
        rec(results, 'neg-reentrant-self-write-own-namespace-ok', report.get('put_own_iid_as_writecap') == 0, report.get('put_own_iid_as_writecap'))
        # Cross-app / sibling denials
        rec(results, 'neg-sibling-read-cap-denied-13', report.get('get_sib_read_cap') == 13, report.get('get_sib_read_cap'))
        rec(results, 'neg-sibling-iid-as-readcap-denied-13', report.get('get_sib_iid_as_readcap') == 13, report.get('get_sib_iid_as_readcap'))
        rec(results, 'neg-sibling-write-cap-denied-13', report.get('put_sib_write_cap') == 13, report.get('put_sib_write_cap'))
        rec(results, 'neg-zero-cap-denied-13', report.get('get_zero_cap') == 13, report.get('get_zero_cap'))
        # Non-appdata ops through the mediator callback are rejected
        rec(results, 'neg-lookup-via-callback-denied-12', report.get('lookup_via_invoke') == 12, report.get('lookup_via_invoke'))
        rec(results, 'neg-register-via-callback-denied-12', report.get('register_via_invoke') == 12, report.get('register_via_invoke'))
        # Private path / raw wire / garbage rejected
        rec(results, 'neg-private-path-raw-opendisk-rejected', report.get('raw_opendisk_via_invoke') == -1, report.get('raw_opendisk_via_invoke'))
        rec(results, 'neg-garbage-invoke-rejected', report.get('garbage_invoke') == -1, report.get('garbage_invoke'))
        # FD guessing denied
        for fd in ['read_fd3', 'read_fd4', 'read_fd99', 'write_fd3', 'write_fd99']:
            rec(results, 'neg-' + fd + '-denied', report.get(fd) == -7, report.get(fd))
        # Victim integrity after attack
        d = srv.send(b'\x01\x03' + put_id(iidA) + put_blob(b'still-ok'), 'victim-A-after-attack')
        rec(results, 'neg-victim-A-still-works-after-attack', d.get('kind') == 'ok' and d.get('body') == b'still-ok', d)
        d = srv.send(b'\x01\x07' + put_id(iidA) + put_blob(b'secret'), 'victim-secret-after-attack')
        rec(results, 'neg-victim-secret-unchanged', d.get('kind') == 'ok' and d.get('body') == b'TOPSECRET-A', d)
        d = srv.send(b'\x01\x07' + put_id(iidA) + put_blob(b'evil'), 'victim-evil-absent')
        rec(results, 'neg-no-sibling-write-contamination', d.get('kind') == 'err' and d.get('code') == 8, d)
    finally:
        if proc.poll() is None: proc.terminate(); proc.wait(timeout=5)

def main():
    check_hashes()
    results = []
    positive(results)
    negative(results)
    bad = [(n, d) for n, ok, d in results if not ok]
    print('\n=== SUMMARY ===', flush=True)
    print(f'TOTAL={len(results)} PASS={len(results) - len(bad)} FAIL={len(bad)}', flush=True)
    for n, d in bad: print('BAD', n, d, flush=True)
    print('ALL_OK=' + str(not bad), flush=True)
    sys.exit(0 if not bad else 1)

if __name__ == '__main__':
    main()

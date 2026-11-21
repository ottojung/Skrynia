import socket, time, sys
PORT=int(sys.argv[1]) if len(sys.argv)>1 else 17401
HOST='127.0.0.1'
def send(payload=b'X', read=True, timeout=6.0):
    s=socket.create_connection((HOST,PORT),timeout=timeout)
    s.sendall((f"POST / HTTP/1.1\r\nHost: x\r\nContent-Length: {len(payload)}\r\nConnection: close\r\n\r\n").encode()+payload)
    if not read: return s
    s.settimeout(timeout); data=b''
    try:
        while True:
            b=s.recv(65536)
            if not b: break
            data+=b
    except socket.timeout: pass
    s.close(); return data
def line(r): return r.split(b'\r\n',1)[0].decode(errors='replace') if r else '<empty>'
print('CONTROL', line(send()))
held=[send(read=False) for _ in range(16)]
time.sleep(3)
print('AFTER_16_NONREADERS', line(send()))
for s in held:
    try: s.close()
    except Exception: pass
time.sleep(2)
print('AFTER_CLOSE', line(send()))

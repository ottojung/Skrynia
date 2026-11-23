import socket, time, sys
PORT=int(sys.argv[1]); HOST='127.0.0.1'
def req(read=True, timeout=6.0):
    s=socket.create_connection((HOST,PORT),timeout=timeout)
    s.sendall(b"POST / HTTP/1.1\r\nHost: x\r\nContent-Length: 1\r\nConnection: close\r\n\r\nX")
    if not read: return s
    s.settimeout(timeout); d=b''
    try:
        while True:
            b=s.recv(65536)
            if not b: break
            d+=b
    except Exception: pass
    s.close(); return d.split(b'\r\n',1)[0].decode(errors='replace') if d else '<empty>'
print('CONTROL', req())
held=[req(read=False) for _ in range(16)]
time.sleep(1)
print('AFTER_16_NONREADERS', req())
# Wait for the response-side watchdog (SKRYNIA_RESPONSE_TOTAL_SECS=4) with clients STILL open.
time.sleep(6)
print('AFTER_WATCHDOG_NO_CLOSE', req())
for s in held:
    try: s.close()
    except Exception: pass
time.sleep(1)
print('AFTER_CLOSE', req())

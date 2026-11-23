import socket, threading, sys
PORT=int(sys.argv[1]) if len(sys.argv)>1 else 17402
HOST='127.0.0.1'
def chunked(total, timeout=20.0):
    s=socket.create_connection((HOST,PORT),timeout=timeout)
    s.sendall(b"POST / HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n")
    left=total; THUNK=1<<20
    def writer():
        nonlocal left
        try:
            while left>0:
                n=min(left,THUNK); b=b'Z'*n
                s.sendall(f'{n:x}\r\n'.encode()+b+b'\r\n'); left-=n
            s.sendall(b'0\r\n\r\n')
        except Exception: pass
    t=threading.Thread(target=writer,daemon=True); t.start()
    s.settimeout(timeout); data=b''
    try:
        while True:
            b=s.recv(65536)
            if not b: break
            data+=b
    except (socket.timeout, ConnectionResetError, ConnectionAbortedError, BrokenPipeError): pass
    s.close()
    head,_,body=data.partition(b'\r\n\r\n')
    return head.split(b'\r\n',1)[0].decode(errors='replace'), len(body)
def content_length(total, timeout=20.0):
    s=socket.create_connection((HOST,PORT),timeout=timeout)
    s.sendall((f"POST / HTTP/1.1\r\nHost: x\r\nContent-Length: {total}\r\nConnection: close\r\n\r\n").encode())
    s.settimeout(timeout); data=b''
    try:
        while True:
            b=s.recv(65536)
            if not b: break
            data+=b
    except (socket.timeout, ConnectionResetError, ConnectionAbortedError, BrokenPipeError): pass
    s.close(); return data.split(b'\r\n',1)[0].decode(errors='replace')
print('OVERSIZE_CHUNKED', chunked(17*1024*1024))
print('OVERSIZE_CONTENTLEN', content_length(17*1024*1024))

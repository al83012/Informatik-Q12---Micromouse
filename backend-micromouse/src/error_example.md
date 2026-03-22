
Send "9" is successful, but is not being received --> Need to check the connection before sending; Send operation is unable to detect disconnects

[2026-03-21T15:24:27Z INFO  comm] SEND "9"
[2026-03-21T15:24:27Z INFO  comm] SEND SUCCESSFUL
[2026-03-21T15:24:27Z WARN  comm] READ 0 bytes --> RECONNECT?
[2026-03-21T15:24:27Z INFO  comm] RECONNECT searching...
[2026-03-21T15:24:27Z INFO  comm] RECONNECT ACCEPTED (192.168.137.106:50405)
[2026-03-21T15:24:28Z INFO  comm] SEND "10"
[2026-03-21T15:24:28Z INFO  comm] SEND SUCCESSFUL
[2026-03-21T15:24:29Z INFO  comm] SEND "11"
[2026-03-21T15:24:29Z INFO  comm] SEND SUCCESSFUL
[2026-03-21T15:24:30Z INFO  comm] SEND "12"
[2026-03-21T15:24:30Z INFO  comm] SEND SUCCESSFUL
[2026-03-21T15:24:31Z INFO  comm] SEND "13"
[2026-03-21T15:24:31Z INFO  comm] SEND SUCCESSFUL
[2026-03-21T15:24:32Z INFO  comm] SEND "14"
[2026-03-21T15:24:32Z INFO  comm] SEND SUCCESSFUL
[2026-03-21T15:24:32Z INFO  comm] READ 18 bytes
[2026-03-21T15:24:32Z INFO  comm] READ
[2026-03-21T15:24:32Z INFO  comm] READ OK ECHO: message(10)

[2026-03-21T15:24:32Z INFO  comm] READ 12 bytes
[2026-03-21T15:24:32Z INFO  comm] READ
[2026-03-21T15:24:32Z INFO  comm] READ OK message(11)


Stabilization works ok, until the connection is reestablished again, at which point the reset connection encounters an IO Error, which somehow leads to the select! macro to not wait for a send, probably because the ping and tick somehow loop

[2026-03-22T14:07:26Z INFO  comm] STABILIZING!!!
[2026-03-22T14:07:26Z INFO  comm] STABILIZING SEND
[2026-03-22T14:07:26Z INFO  comm] HANDLE PING
[2026-03-22T14:07:27Z INFO  comm] PONG TOO LATE: 16.0293068s
[2026-03-22T14:07:27Z INFO  comm] STABILIZING!!!
[2026-03-22T14:07:27Z INFO  comm] STABILIZING READ
[2026-03-22T14:07:27Z INFO  comm] HANDLE READ (Err(Io(Os { code: 10054, kind: ConnectionReset, message: "An existing connection was forcibly closed by the remote host." })))
[2026-03-22T14:07:27Z INFO  comm] HANDLE RECOVERABLE? ATTEMPT
[2026-03-22T14:07:27Z INFO  comm] PONG TOO LATE: 16.4858273s
[2026-03-22T14:07:27Z INFO  comm] STABILIZING!!!
[2026-03-22T14:07:27Z INFO  comm] STABILIZING READ
[2026-03-22T14:07:27Z INFO  comm] PONG TOO LATE: 16.6894631s
[2026-03-22T14:07:27Z INFO  comm] STABILIZING!!!
[2026-03-22T14:07:27Z INFO  comm] STABILIZING READ
[2026-03-22T14:07:28Z INFO  comm] PONG TOO LATE: 16.8968317s
[2026-03-22T14:07:28Z INFO  comm] STABILIZING!!!
[2026-03-22T14:07:28Z INFO  comm] STABILIZING SEND
[2026-03-22T14:07:28Z INFO  comm] HANDLE PING
[2026-03-22T14:07:28Z INFO  comm] HANDLE RECOVERABLE? ATTEMPT
[2026-03-22T14:07:28Z INFO  comm] PONG TOO LATE: 17.100739s
[2026-03-22T14:07:28Z INFO  comm] STABILIZING!!!
[2026-03-22T14:07:28Z INFO  comm] STABILIZING READ



It seems that our reconnect method is not fully working --> it errors out before it can properly reset --> After "Opening new ws", we should se "Finished Reconnect"

[2026-03-22T14:34:34Z INFO  comm] HANDLE PING
[2026-03-22T14:34:34Z ERROR comm] PONG TOO LATE: 22.9856607s
[2026-03-22T14:34:34Z WARN  comm] STABILIZING!!!
[2026-03-22T14:34:34Z INFO  comm] STABILIZING READ
[2026-03-22T14:34:34Z INFO  comm] HANDLE READ (Err(Io(Os { code: 10054, kind: ConnectionReset, message: "An existing connection was forcibly closed by the remote host." })))
[2026-03-22T14:34:34Z INFO  comm] HANDLE RECOVERABLE? ATTEMPT
[2026-03-22T14:34:34Z WARN  comm] IO ERROR
[2026-03-22T14:34:34Z INFO  comm] HANDLE ERROR? (An existing connection was forcibly closed by the remote host. (os error 10054))
[2026-03-22T14:34:34Z INFO  comm] RECOVERABLE --> RECONNECT
[2026-03-22T14:34:34Z INFO  comm] RECONNECT searching...
[2026-03-22T14:34:34Z INFO  comm] RECONNECT ACCEPTED (192.168.137.222:64299)
[2026-03-22T14:34:34Z INFO  comm] CLOSING OLD WS
[2026-03-22T14:34:34Z INFO  comm] OPENING NEW WS
[2026-03-22T14:34:35Z ERROR comm] PONG TOO LATE: 23.2056877s
[2026-03-22T14:34:35Z WARN  comm] STABILIZING!!!
[2026-03-22T14:34:35Z INFO  comm] STABILIZING READ
[2026-03-22T14:34:35Z ERROR comm] PONG TOO LATE: 23.4120569s



When resetting the entire listener, it seems to work


[2026-03-22T15:01:26Z INFO  comm] HANDLE READ (Ok(Pong(b"-")))
[2026-03-22T15:01:26Z INFO  comm] RECV PONG (latency = 239.1898ms, period = 109.7855ms)
[2026-03-22T15:01:26Z INFO  comm] STABLE TICK
[2026-03-22T15:01:27Z INFO  comm] HANDLE PING
[2026-03-22T15:01:27Z INFO  comm] STABLE TICK
[2026-03-22T15:01:28Z INFO  comm] HANDLE PING
[2026-03-22T15:01:28Z ERROR comm] PONG TOO LATE: 1.8779055s
[2026-03-22T15:01:28Z WARN  comm] STABILIZING!!!
[2026-03-22T15:01:29Z INFO  comm] STABILIZING SEND
[2026-03-22T15:01:29Z INFO  comm] HANDLE PING
[2026-03-22T15:01:29Z ERROR comm] PONG TOO LATE: 2.9628486s
[2026-03-22T15:01:29Z WARN  comm] STABILIZING!!!
[2026-03-22T15:01:29Z INFO  comm] STABILIZING READ
[2026-03-22T15:01:29Z INFO  comm] HANDLE READ (Err(Io(Os { code: 10054, kind: ConnectionReset, message: "An existing connection was forcibly closed by the remote host." })))
[2026-03-22T15:01:30Z INFO  comm] HANDLE RECOVERABLE? ATTEMPT
[2026-03-22T15:01:30Z WARN  comm] IO ERROR
[2026-03-22T15:01:30Z INFO  comm] HANDLE ERROR? (An existing connection was forcibly closed by the remote host. (os error 10054))
[2026-03-22T15:01:30Z INFO  comm] RECOVERABLE --> RECONNECT
[2026-03-22T15:01:30Z INFO  comm] RECONNECT searching...
[2026-03-22T15:01:32Z INFO  comm] RECONNECT ACCEPTED (192.168.137.222:62350)
[2026-03-22T15:01:32Z INFO  comm] CLOSING OLD WS
[2026-03-22T15:01:32Z INFO  comm] OPENING NEW WS
[2026-03-22T15:01:32Z INFO  comm] NEW WS = Ok(WebSocketStream { inner: WebSocket { socket: AllowStd { inner: TcpStream { addr: 192.168.137.1:9001, peer: 192.168.137.222:62350, socket: 328 }
, write_waker_proxy: WakerProxy { read_waker: AtomicWaker, write_waker: AtomicWaker }, read_waker_proxy: WakerProxy { read_waker: AtomicWaker, write_waker: AtomicWaker } }, context: WebSock
etContext { role: Server, frame: FrameCodec { in_buffer: b"", in_buf_max_read: 131072, out_buffer: [], max_out_buffer_len: 18446744073709551615, out_buffer_write_len: 131072, header: None }
, state: Active, incomplete: None, additional_send: None, unflushed_additional: false, config: WebSocketConfig { read_buffer_size: 131072, write_buffer_size: 131072, max_write_buffer_size: 
18446744073709551615, max_message_size: Some(67108864), max_frame_size: Some(16777216), accept_unmasked_frames: false } } }, closing: false, ended: false, ready: true })
[2026-03-22T15:01:32Z INFO  comm] FINISHED RECONNECT
[2026-03-22T15:01:32Z INFO  comm] STABLE TICK
[2026-03-22T15:01:32Z INFO  comm] HANDLE PING
[2026-03-22T15:01:32Z INFO  comm] STABLE TICK
[2026-03-22T15:01:32Z INFO  comm] HANDLE READ (Ok(Text(Utf8Bytes(b"Hi Server!"))))
[2026-03-22T15:01:32Z INFO  comm] STABLE TICK
[2026-03-22T15:01:32Z INFO  comm] HANDLE PING
[2026-03-22T15:01:32Z INFO  comm] STABLE TICK
[2026-03-22T15:01:32Z INFO  comm] HANDLE PING
[2026-03-22T15:01:32Z INFO  comm] STABLE TICK
[2026-03-22T15:01:32Z INFO  comm] HANDLE READ (Ok(Ping(b"")))
[2026-03-22T15:01:32Z INFO  comm] HANDLE PING
[2026-03-22T15:01:32Z INFO  comm] STABLE TICK
[2026-03-22T15:01:32Z INFO  comm] HANDLE READ (Ok(Pong(b"1")))
[2026-03-22T15:01:32Z INFO  comm] RECV PONG (latency = 110.6239ms, period = 762.9949ms)
[2026-03-22T15:01:33Z INFO  comm] STABLE TICK
[2026-03-22T15:01:33Z INFO  comm] HANDLE READ (Ok(Pong(b"1")))
[2026-03-22T15:01:33Z INFO  comm] RECV PONG (latency = 223.6645ms, period = 113.0403ms)
[2026-03-22T15:01:33Z INFO  comm] STABLE TICK
[2026-03-22T15:01:33Z INFO  comm] HANDLE PING
[2026-03-22T15:01:33Z INFO  comm] STABLE TICK
[2026-03-22T15:01:33Z INFO  comm] HANDLE READ (Ok(Pong(b"2")))
[2026-03-22T15:01:33Z INFO  comm] RECV PONG (latency = 112.541ms, period = 226.3278ms)
[2026-03-22T15:01:33Z INFO  comm] STABLE TICK
[2026-03-22T15:01:33Z INFO  comm] HANDLE READ (Ok(Pong(b"2")))
[2026-03-22T15:01:33Z INFO  comm] RECV PONG (latency = 225.617ms, period = 113.0758ms)
[2026-03-22T15:01:33Z INFO  comm] STABLE TICK
[2026-03-22T15:01:33Z INFO  comm] HANDLE READ (Ok(Pong(b"3")))
[2026-03-22T15:01:33Z INFO  comm] RECV PONG (latency = 337.6771ms, period = 112.0599ms)
[2026-03-22T15:01:33Z INFO  comm] STABLE TICK
[2026-03-22T15:01:33Z INFO  comm] HANDLE READ (Ok(Pong(b"3")))
[2026-03-22T15:01:33Z INFO  comm] RECV PONG (latency = 449.0465ms, period = 111.369ms)
[2026-03-22T15:01:33Z INFO  comm] STABLE TICK
[2026-03-22T15:01:33Z INFO  comm] HANDLE READ (Ok(Pong(b"4")))
[2026-03-22T15:01:33Z INFO  comm] RECV PONG (latency = 561.5509ms, period = 112.5041ms)
[2026-03-22T15:01:33Z INFO  comm] STABLE TICK
[2026-03-22T15:01:33Z INFO  comm] HANDLE READ (Ok(Pong(b"4")))
[2026-03-22T15:01:33Z INFO  comm] RECV PONG (latency = 673.9076ms, period = 112.3565ms)
[2026-03-22T15:01:34Z INFO  comm] STABLE TICK
[2026-03-22T15:01:34Z INFO  comm] HANDLE READ (Ok(Pong(b"5")))
[2026-03-22T15:01:34Z INFO  comm] RECV PONG (latency = 788.0447ms, period = 114.1368ms)
[2026-03-22T15:01:34Z INFO  comm] STABLE TICK
[2026-03-22T15:01:34Z INFO  comm] HANDLE READ (Ok(Pong(b"5")))
[2026-03-22T15:01:34Z INFO  comm] RECV PONG (latency = 900.9935ms, period = 112.9486ms)
[2026-03-22T15:01:34Z INFO  comm] STABLE TICK
[2026-03-22T15:01:34Z INFO  comm] HANDLE PING
[2026-03-22T15:01:34Z INFO  comm] STABLE TICK
[2026-03-22T15:01:34Z INFO  comm] HANDLE READ (Ok(Pong(b"6")))
[2026-03-22T15:01:34Z INFO  comm] RECV PONG (latency = 175.8986ms, period = 289.7041ms)
[2026-03-22T15:01:34Z INFO  comm] STABLE TICK
[2026-03-22T15:01:34Z INFO  comm] HANDLE READ (Ok(Pong(b"6")))
[2026-03-22T15:01:34Z INFO  comm] RECV PONG (latency = 291.1683ms, period = 115.2695ms)
[2026-03-22T15:01:34Z INFO  comm] STABLE TICK
[2026-03-22T15:01:35Z INFO  comm] HANDLE PING
[2026-03-22T15:01:35Z INFO  comm] STABLE TICK
[2026-03-22T15:01:35Z INFO  comm] HANDLE READ (Ok(Pong(b"7")))
[2026-03-22T15:01:35Z INFO  comm] RECV PONG (latency = 246.5759ms, period = 907.8102ms)
[2026-03-22T15:01:35Z INFO  comm] STABLE TICK
[2026-03-22T15:01:35Z INFO  comm] HANDLE READ (Ok(Pong(b"7")))
[2026-03-22T15:01:35Z INFO  comm] RECV PONG (latency = 364.0731ms, period = 117.4969ms)
[2026-03-22T15:01:35Z INFO  comm] STABLE TICK
[2026-03-22T15:01:36Z INFO  comm] HANDLE PING
[2026-03-22T15:01:36Z INFO  comm] STABLE TICK
[2026-03-22T15:01:37Z INFO  comm] HANDLE PING
[2026-03-22T15:01:37Z ERROR comm] PONG TOO LATE: 1.7529033s
[2026-03-22T15:01:37Z WARN  comm] STABILIZING!!!
[2026-03-22T15:01:38Z INFO  comm] STABILIZING SEND
[2026-03-22T15:01:38Z INFO  comm] HANDLE PING
[2026-03-22T15:01:38Z ERROR comm] PONG TOO LATE: 2.8401951s
[2026-03-22T15:01:38Z WARN  comm] STABILIZING!!!
[2026-03-22T15:01:39Z INFO  comm] STABILIZING SEND
[2026-03-22T15:01:39Z INFO  comm] HANDLE PING
[2026-03-22T15:01:39Z ERROR comm] PONG TOO LATE: 3.8409634s
[2026-03-22T15:01:39Z WARN  comm] STABILIZING!!!
[2026-03-22T15:01:40Z INFO  comm] STABILIZING SEND
[2026-03-22T15:01:40Z INFO  comm] HANDLE PING
[2026-03-22T15:01:40Z ERROR comm] PONG TOO LATE: 4.8459997s
[2026-03-22T15:01:40Z WARN  comm] STABILIZING!!!
[2026-03-22T15:01:40Z INFO  comm] STABILIZING READ
[2026-03-22T15:01:40Z INFO  comm] HANDLE READ (Err(Io(Os { code: 10054, kind: ConnectionReset, message: "An existing connection was forcibly closed by the remote host." })))
[2026-03-22T15:01:40Z INFO  comm] HANDLE RECOVERABLE? ATTEMPT
[2026-03-22T15:01:40Z WARN  comm] IO ERROR
[2026-03-22T15:01:40Z INFO  comm] HANDLE ERROR? (An existing connection was forcibly closed by the remote host. (os error 10054))
[2026-03-22T15:01:40Z INFO  comm] RECOVERABLE --> RECONNECT
[2026-03-22T15:01:40Z INFO  comm] RECONNECT searching...
[2026-03-22T15:01:40Z INFO  comm] RECONNECT ACCEPTED (192.168.137.222:50055)
[2026-03-22T15:01:40Z INFO  comm] CLOSING OLD WS
[2026-03-22T15:01:40Z INFO  comm] OPENING NEW WS
[2026-03-22T15:01:40Z INFO  comm] NEW WS = Ok(WebSocketStream { inner: WebSocket { socket: AllowStd { inner: TcpStream { addr: 192.168.137.1:9001, peer: 192.168.137.222:50055, socket: 356 }
, write_waker_proxy: WakerProxy { read_waker: AtomicWaker, write_waker: AtomicWaker }, read_waker_proxy: WakerProxy { read_waker: AtomicWaker, write_waker: AtomicWaker } }, context: WebSock
etContext { role: Server, frame: FrameCodec { in_buffer: b"", in_buf_max_read: 131072, out_buffer: [], max_out_buffer_len: 18446744073709551615, out_buffer_write_len: 131072, header: None }
, state: Active, incomplete: None, additional_send: None, unflushed_additional: false, config: WebSocketConfig { read_buffer_size: 131072, write_buffer_size: 131072, max_write_buffer_size: 
18446744073709551615, max_message_size: Some(67108864), max_frame_size: Some(16777216), accept_unmasked_frames: false } } }, closing: false, ended: false, ready: true })
[2026-03-22T15:01:40Z INFO  comm] FINISHED RECONNECT
[2026-03-22T15:01:40Z INFO  comm] STABLE TICK
[2026-03-22T15:01:40Z INFO  comm] HANDLE READ (Ok(Text(Utf8Bytes(b"Hi Server!"))))
[2026-03-22T15:01:41Z INFO  comm] STABLE TICK
[2026-03-22T15:01:41Z INFO  comm] HANDLE READ (Ok(Ping(b"")))
[2026-03-22T15:01:41Z INFO  comm] HANDLE PING
[2026-03-22T15:01:41Z INFO  comm] STABLE TICK
[2026-03-22T15:01:41Z INFO  comm] HANDLE PING
[2026-03-22T15:01:41Z INFO  comm] STABLE TICK
[2026-03-22T15:01:41Z INFO  comm] HANDLE READ (Ok(Pong(b"=")))
[2026-03-22T15:01:41Z INFO  comm] RECV PONG (latency = 112.3238ms, period = 565.2984ms)

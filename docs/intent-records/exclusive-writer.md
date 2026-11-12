$id-4248198621776748
title: Kernel-enforced single writer
date: 2026/11/12
source: @ottojung
kind: requirement

Exactly one Skrynia server process may own a data directory. On startup it
obtains an exclusive, non-blocking OS advisory lock on the persistent private
state/writer.lock file before loading grants, recovering journals, or serving
requests. A competing writer fails startup without altering object or grant
state. The kernel releases ownership on process exit, including abrupt death.
Normal shutdown releases ownership after outstanding asynchronous push state
mutations have drained. The lock file remains in place across restarts.
Operators must store the whole data directory on a filesystem providing
reliable flock coordination among all possible writers.

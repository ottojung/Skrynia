$id-0203518065789149
title: Hierarchical public object identifiers
date: 2026/11/16
source: @ottojung
kind: requirement

Skrynia accepts slash-separated object identifiers so clients can upload
static application directory trees over the existing authenticated HTTP
object protocol without flattening layouts. A key remains an independently
addressable public byte object supporting atomic create, replace, delete,
and content-hash CAS. Reject empty segments, dot/dotdot traversal, unsafe
URL decoding, symlinks and nonregular filesystem entries. Location matchers,
quota counts, listings, restart recovery, and public GET work across nested
objects. The public store path mirrors the key; directories are not objects.
The same path cannot simultaneously designate a file and a parent directory.

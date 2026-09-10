$id-4070814447118336
title: Atomic CRUD operations
date: 2026/09/10
source: @ottojung
kind: requirement

Public store operations are get, create, put, delete. Each operation on one object is atomic. create must atomically fail if key exists. put atomically replaces. delete atomically removes. No partial file contents may become visible.

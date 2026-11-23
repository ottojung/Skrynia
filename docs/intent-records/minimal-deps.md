$id-6214634120670208
title: Small kernel dependency graph
date: 2026/11/12
source: @ottojung
kind: requirement

Implement the native kernel and its tests in Rust, preferring standard-library
facilities and a minimal dependency graph. Operator-supplied userland programs
are built separately and are not native kernel components.

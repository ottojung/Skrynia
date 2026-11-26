$id-1891053258346579
title: Open and sync the storage root directory
date: 2026/11/26
source: @ottojung
kind: requirement

open_directory(".") must return a directory descriptor for the selected,
authorized storage root itself. sync on that descriptor requests directory
durability using the same semantics as other directory descriptors, so
software can durably commit creation, deletion and rename of top-level
entries. No new kernel function is needed.

This narrow path rule must not relax filesystem confinement, authorize a
different root, allow parent traversal, or change the path rules for
unrelated operations. Resolve the root using the normal authorization and
confined directory-descriptor machinery.

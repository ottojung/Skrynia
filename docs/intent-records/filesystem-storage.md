$id-5943420546985984
title: Filesystem backed storage
date: 2026/09/10
source: @ottojung
kind: requirement

Storage should be actual inspectable files on disk, naturally corresponding to keys. Design a safe mapping/canonical key policy preventing traversal/aliasing. Keep internal metadata separate/reserved. Avoid exposing internal files via object keys.

$id-9183047265512048
title: Standalone HTTP reads
date: 2026/11/10
source: @ottojung
kind: requirement

The Node.js server serves public object GET requests. nginx or another configured static server may serve public object files independently.\n\nStandalone object reads support ETag 304 revalidation, HEAD, and single-byte-range GET requests.\n
$id-4521391348921397
title: Versioned ETags
date: 2026/11/10
source: @ottojung
kind: requirement

GET returns an opaque strong ETag. Every successful mutation advances the monotonically increasing global version, including identical-byte replacement and deletion/recreation. PUT and DELETE support If-Match. ETags use the nginx-compatible mtime-seconds/size form.

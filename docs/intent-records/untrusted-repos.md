$id-4285944656329984
title: Untrusted repo safety
date: 2026/09/10
source: @ottojung
kind: requirement

Git/source and builder behavior must be safe with untrusted app repositories: disposable workspace, bounded/controlled output, no secret injection, no privileged host mounts. Do not over-engineer a sandbox beyond the shared container model specified.

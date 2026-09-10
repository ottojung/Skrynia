$id-4285944656329984
title: Untrusted repo safety
date: 2026/09/10
source: @ottojung
kind: requirement

Git/source repositories are untrusted in the ordinary sense: disposable workspace, bounded/controlled output, no secret injection, no privileged host mounts. Builds run in throwaway containers and may access the network; security isolation is not a build-time goal.

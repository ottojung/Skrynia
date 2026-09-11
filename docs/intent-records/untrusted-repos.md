$id-4285944656329984
title: Untrusted repo safety
date: 2026/09/11
source: @ottojung
kind: requirement

Git/source repositories are untrusted in the ordinary sense: use a disposable workspace and bounded/controlled build output. Repository cloning may use read-only SSH credentials supplied to the Skrynia management process. Those credentials and other secrets are not injected into the app build container, and the build container receives no privileged host mounts. Builds run in throwaway containers and may access the network; security isolation is not a build-time goal.

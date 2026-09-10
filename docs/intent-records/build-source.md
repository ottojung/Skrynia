$id-8381035660795904
title: Apps built from exact git repo commit subdirectory
date: 2026/09/10
source: @ottojung
kind: requirement

Apps are built/deployed from an exact Git repository + commit + subdirectory. Monorepos are ordinary repos: keep the whole checkout available but run make build from the app subdirectory so it may use ../shared etc.

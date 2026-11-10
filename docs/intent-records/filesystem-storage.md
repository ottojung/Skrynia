$id-5943420546985984
title: Filesystem object storage
date: 2026/11/10
source: @ottojung
kind: requirement

Public objects are exact files under STORE_DIR/{key}. Private grants, subscriptions, journal and staging remain outside the public tree. Names use ASCII letters, digits, underscores, dots, excluding the exact names `.` and `..`. Slashes are forbidden.

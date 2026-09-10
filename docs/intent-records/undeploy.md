$id-0867036237408256
title: Undeploy removes everything by default
date: 2026/09/10
source: @ottojung
kind: requirement

Add an undeploy CLI command. By default undeploy must remove the deployed website/release state AND delete that namespace and all of its stored data. This intentionally supersedes the earlier idea that app removal should preserve durable data. If a preserve-data option is supported, it must be explicit and non-default; do not add it unless it stays simple.
